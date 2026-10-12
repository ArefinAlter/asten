//! Relations and genealogy (§9.9): acyclicity with Tarjan's algorithm,
//! exclusive relations over overlapping intervals, and parent-age constraints.

use std::collections::{BTreeMap, HashMap};

use asten_diagnostics::{Repair, RepairKind, Tier, Violation, catalog};
use asten_ir::{EntityId, EventKind, Fact, PredicateId, Status, StoryIr, TimeBounds, Value};

use crate::time::TimeTable;
use crate::util::{
    event_statement, fact_statement, fluent_label, joint_confidence, name_of, pairing_of,
};

struct Edge<'a> {
    from: &'a EntityId,
    to: &'a EntityId,
    fact: &'a Fact,
}

/// CANON-E501, E502, E503.
#[must_use]
pub fn check(ir: &StoryIr, times: &TimeTable) -> Vec<Violation> {
    let mut violations = Vec::new();
    let relation_facts: Vec<&Fact> = ir
        .facts
        .iter()
        .filter(|fact| fact.status.is_active() && matches!(fact.fluent.value, Value::Entity { .. }))
        .collect();

    // Normalise inverses: child_of(a, b) is parent_of(b, a).
    let mut edges_by_predicate: BTreeMap<PredicateId, Vec<Edge<'_>>> = BTreeMap::new();
    for fact in &relation_facts {
        let Value::Entity { id: object } = &fact.fluent.value else {
            continue;
        };
        let predicate = &fact.fluent.predicate;
        let sig = ir.predicate(predicate);
        let canonical = sig
            .and_then(|sig| sig.relation.inverse.as_ref())
            .filter(|inverse| inverse.0 < predicate.0 && ir.predicate(inverse).is_some())
            .cloned();
        match canonical {
            Some(inverse) => edges_by_predicate.entry(inverse).or_default().push(Edge {
                from: object,
                to: &fact.fluent.subject,
                fact,
            }),
            None => edges_by_predicate
                .entry(predicate.clone())
                .or_default()
                .push(Edge {
                    from: &fact.fluent.subject,
                    to: object,
                    fact,
                }),
        }
    }

    for (predicate, edges) in &edges_by_predicate {
        let Some(sig) = ir.predicate(predicate) else {
            continue;
        };
        // E501: cycles in acyclic relations.
        if sig.relation.acyclic {
            for cycle in cycles(edges) {
                let conflict: Vec<_> = cycle
                    .iter()
                    .map(|edge| fact_statement(ir, edge.fact))
                    .collect();
                let names: Vec<String> = cycle.iter().map(|edge| name_of(ir, edge.from)).collect();
                violations.push(Violation {
                    code: catalog::GENEALOGY_CYCLE.into(),
                    title: format!(
                        "{} ends up as their own {}.",
                        names.first().cloned().unwrap_or_default(),
                        sig.label
                    ),
                    primary: conflict
                        .iter()
                        .find_map(|statement| statement.location.clone()),
                    related: Vec::new(),
                    confidence: joint_confidence(&conflict),
                    pairing: pairing_of(&conflict),
                    conflict,
                    tier: Tier::Must,
                    repairs: cycle
                        .iter()
                        .map(|edge| {
                            Repair::new(
                                RepairKind::DeprecateFact {
                                    fact: edge.fact.id.0.clone(),
                                },
                                format!(
                                    "Retire \u{201c}{}\u{201d}",
                                    fluent_label(ir, &edge.fact.fluent)
                                ),
                                if edge.fact.status == Status::Settled {
                                    4.0
                                } else {
                                    1.0
                                },
                            )
                        })
                        .chain([Repair::keep()])
                        .collect(),
                    entity: cycle.first().map(|edge| edge.from.clone()),
                    details: format!(
                        "The relation {} was declared acyclic; these statements form a loop.",
                        sig.label
                    ),
                    severity_override: None,
                });
            }
        }
        // E502: exclusive relations overlapping in time.
        if sig.relation.exclusive {
            let mut by_subject: HashMap<&EntityId, Vec<&Edge<'_>>> = HashMap::new();
            for edge in edges {
                by_subject.entry(edge.from).or_default().push(edge);
                if sig.relation.symmetric {
                    by_subject.entry(edge.to).or_default().push(edge);
                }
            }
            for (subject, group) in by_subject {
                for (index, left) in group.iter().enumerate() {
                    for right in group.iter().skip(index + 1) {
                        if left.fact.id == right.fact.id {
                            continue;
                        }
                        if partner(left, subject) == partner(right, subject) {
                            continue;
                        }
                        let Some(_) = left.fact.valid.intersect(&right.fact.valid) else {
                            continue;
                        };
                        let conflict = vec![
                            fact_statement(ir, left.fact),
                            fact_statement(ir, right.fact),
                        ];
                        violations.push(Violation {
                            code: catalog::EXCLUSIVE_OVERLAP.into(),
                            title: format!("{} has two {} at once.", name_of(ir, subject), plural(&sig.label)),
                            primary: conflict.iter().find_map(|statement| statement.location.clone()),
                            related: Vec::new(),
                            confidence: joint_confidence(&conflict),
                            pairing: pairing_of(&conflict),
                            conflict,
                            tier: Tier::Must,
                            repairs: vec![
                                Repair::new(RepairKind::SetValidity { fact: left.fact.id.0.clone(), from_tick: None, to_tick: right.fact.valid.lo.map(|lo| lo - 1) }, "Give the first one an end", 1.0),
                                Repair::new(RepairKind::DeprecateFact { fact: right.fact.id.0.clone() }, "Retire the second", 2.0),
                                Repair::suppress(),
                                Repair::keep(),
                            ],
                            entity: Some(subject.clone()),
                            details: "Declared exclusive in the predicate's signature. A world rule can lift it.".into(),
                            severity_override: None,
                        });
                    }
                }
            }
        }
    }

    // E503: parent age against births.
    if let Some(min_age) = ir.settings.min_parent_age_ticks {
        let births: HashMap<&EntityId, (TimeBounds, &asten_ir::Event)> = ir
            .events
            .iter()
            .filter(|event| event.status.is_active() && matches!(event.kind, EventKind::Born))
            .filter_map(|event| {
                event
                    .subject
                    .as_ref()
                    .map(|subject| (subject, (times.event(&event.id), event)))
            })
            .collect();
        for (predicate, edges) in &edges_by_predicate {
            if predicate.0 != "parent_of" {
                continue;
            }
            for edge in edges {
                let (Some((parent_born, parent_event)), Some((child_born, child_event))) =
                    (births.get(edge.from), births.get(edge.to))
                else {
                    continue;
                };
                // Certainly too young: child.hi − parent.lo < min_age.
                let tier = match (child_born.hi, parent_born.lo) {
                    (Some(child_hi), Some(parent_lo)) if child_hi - parent_lo < min_age => {
                        Tier::Must
                    }
                    _ => match (child_born.lo, parent_born.hi) {
                        (Some(child_lo), Some(parent_hi)) if child_lo - parent_hi < min_age => {
                            Tier::May
                        }
                        _ => continue,
                    },
                };
                let conflict = vec![
                    fact_statement(ir, edge.fact),
                    event_statement(ir, parent_event, parent_born),
                    event_statement(ir, child_event, child_born),
                ];
                violations.push(Violation {
                    code: catalog::PARENT_AGE.into(),
                    title: format!(
                        "{} is too young to be {}'s parent.",
                        name_of(ir, edge.from),
                        name_of(ir, edge.to)
                    ),
                    primary: conflict
                        .iter()
                        .find_map(|statement| statement.location.clone()),
                    related: Vec::new(),
                    confidence: joint_confidence(&conflict),
                    pairing: pairing_of(&conflict),
                    conflict,
                    tier,
                    repairs: vec![
                        Repair::new(
                            RepairKind::MoveEvent {
                                event: parent_event.id.0.clone(),
                                lo: None,
                                hi: child_born.lo.map(|lo| lo - min_age),
                            },
                            "Move the parent's birth earlier",
                            1.0,
                        ),
                        Repair::new(
                            RepairKind::MoveEvent {
                                event: child_event.id.0.clone(),
                                lo: parent_born.hi.map(|hi| hi + min_age),
                                hi: None,
                            },
                            "Move the child's birth later",
                            1.0,
                        ),
                        Repair::suppress(),
                        Repair::keep(),
                    ],
                    entity: Some(edge.from.clone()),
                    details: String::new(),
                    severity_override: None,
                });
            }
        }
    }
    violations
}

fn partner<'a>(edge: &Edge<'a>, subject: &EntityId) -> &'a EntityId {
    if edge.from == subject {
        edge.to
    } else {
        edge.from
    }
}

fn plural(label: &str) -> String {
    if label.ends_with('s') {
        label.to_owned()
    } else {
        format!("{label}s")
    }
}

/// Tarjan's strongly connected components; returns one edge cycle per
/// non-trivial component (or self-loop).
fn cycles<'a, 'e>(edges: &'e [Edge<'a>]) -> Vec<Vec<&'e Edge<'a>>> {
    let mut nodes: Vec<&EntityId> = Vec::new();
    let mut index_of: HashMap<&EntityId, usize> = HashMap::new();
    for edge in edges {
        for node in [edge.from, edge.to] {
            if !index_of.contains_key(node) {
                index_of.insert(node, nodes.len());
                nodes.push(node);
            }
        }
    }
    let n = nodes.len();
    let mut adjacency: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for (edge_index, edge) in edges.iter().enumerate() {
        adjacency[index_of[edge.from]].push((index_of[edge.to], edge_index));
    }
    // Iterative Tarjan.
    let mut index = vec![usize::MAX; n];
    let mut low = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut component = vec![usize::MAX; n];
    let mut counter = 0usize;
    let mut components = 0usize;
    for start in 0..n {
        if index[start] != usize::MAX {
            continue;
        }
        let mut work: Vec<(usize, usize)> = vec![(start, 0)];
        index[start] = counter;
        low[start] = counter;
        counter += 1;
        stack.push(start);
        on_stack[start] = true;
        while let Some(&mut (node, ref mut next)) = work.last_mut() {
            if *next < adjacency[node].len() {
                let (target, _) = adjacency[node][*next];
                *next += 1;
                if index[target] == usize::MAX {
                    index[target] = counter;
                    low[target] = counter;
                    counter += 1;
                    stack.push(target);
                    on_stack[target] = true;
                    work.push((target, 0));
                } else if on_stack[target] {
                    low[node] = low[node].min(index[target]);
                }
            } else {
                work.pop();
                if let Some(&(parent, _)) = work.last() {
                    low[parent] = low[parent].min(low[node]);
                }
                if low[node] == index[node] {
                    loop {
                        let popped = stack.pop().expect("stack");
                        on_stack[popped] = false;
                        component[popped] = components;
                        if popped == node {
                            break;
                        }
                    }
                    components += 1;
                }
            }
        }
    }
    // Edges inside a component form the cycle set; a component is cyclic when
    // it has more than one node or a self-loop.
    let mut out: Vec<Vec<&Edge<'a>>> = Vec::new();
    for component_id in 0..components {
        let members: Vec<usize> = (0..n)
            .filter(|node| component[*node] == component_id)
            .collect();
        let inner: Vec<&Edge<'a>> = edges
            .iter()
            .filter(|edge| {
                component[index_of[edge.from]] == component_id
                    && component[index_of[edge.to]] == component_id
            })
            .collect();
        if (members.len() > 1 || inner.iter().any(|edge| edge.from == edge.to)) && !inner.is_empty()
        {
            out.push(inner);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use asten_ir::{
        Cardinality, Context, Entity, FactId, Fluent, Mutability, PredicateSig, RelationProps,
        Sort, ValueDomain,
    };

    fn relation(subject: &str, predicate: &str, object: &str) -> Fact {
        Fact {
            id: FactId::new(format!("{subject}-{predicate}-{object}")),
            fluent: Fluent {
                predicate: PredicateId::new(predicate),
                subject: EntityId::new(subject),
                value: Value::Entity {
                    id: EntityId::new(object),
                },
            },
            context: Context::World,
            valid: TimeBounds::UNBOUNDED,
            status: Status::Settled,
            provenance: asten_ir::Provenance::default(),
            confidence: 1.0,
            anchor: None,
            settled_at: None,
        }
    }

    fn ir() -> StoryIr {
        let mut ir = StoryIr::default();
        for name in ["a", "b", "c"] {
            ir.entities.push(Entity {
                id: EntityId::new(name),
                sort: Sort::Character,
                name: name.to_uppercase(),
                aliases: Vec::new(),
                introduced_at: None,
            });
        }
        ir.predicates.push(PredicateSig {
            id: PredicateId::new("parent_of"),
            label: "parent".into(),
            subject_sorts: vec![],
            domain: ValueDomain::Entity {
                sort: Some(Sort::Character),
            },
            cardinality: Cardinality::Multi,
            mutability: Mutability::Immutable,
            relation: RelationProps {
                acyclic: true,
                transitive: false,
                symmetric: false,
                inverse: Some(PredicateId::new("child_of")),
                exclusive: false,
            },
            cues: vec![],
            pack: "relations".into(),
        });
        ir.predicates.push(PredicateSig {
            id: PredicateId::new("married_to"),
            label: "spouse".into(),
            subject_sorts: vec![],
            domain: ValueDomain::Entity {
                sort: Some(Sort::Character),
            },
            cardinality: Cardinality::Multi,
            mutability: Mutability::Mutable,
            relation: RelationProps {
                symmetric: true,
                exclusive: true,
                ..Default::default()
            },
            cues: vec![],
            pack: "relations".into(),
        });
        ir
    }

    #[test]
    fn finds_genealogy_cycles() {
        let mut ir = ir();
        ir.facts.push(relation("a", "parent_of", "b"));
        ir.facts.push(relation("b", "parent_of", "c"));
        ir.facts.push(relation("c", "parent_of", "a"));
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "CANON-E501");
        assert_eq!(violations[0].conflict.len(), 3);
    }

    #[test]
    fn exclusive_relations_overlapping_in_time() {
        let mut ir = ir();
        let mut first = relation("a", "married_to", "b");
        first.valid = TimeBounds::from(10);
        let mut second = relation("a", "married_to", "c");
        second.valid = TimeBounds::from(20);
        ir.facts.push(first);
        ir.facts.push(second);
        let (times, _) = crate::resolve_times(&ir);
        let violations = check(&ir, &times);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, "CANON-E502");
        ir.facts[0].valid = TimeBounds::between(10, 19);
        assert!(check(&ir, &times).is_empty());
    }
}
