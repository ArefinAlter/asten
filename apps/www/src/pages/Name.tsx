import { Link } from "react-router-dom";
import { Frieze } from "../art/Friezes";
import { LunarEmblem } from "../art/Night";
import { PaletteMark } from "../art/Palette";
import { Scales } from "../art/Scales";
import { Thoth } from "../art/Thoth";
import { Arrow } from "../components/Icons";
import { Answer, CtaBand, PageHero, Related } from "../components/Page";
import { CartoucheName } from "../demos/CartoucheName";

export function Name() {
  return (
    <>
      <PageHero
        title="Asten is another name for Thoth."
        lede="Thoth was the Egyptian god of writing: inventor of hieroglyphs, keeper of the gods’ records, reckoner of time by the moon, and the scribe who wrote down the verdict at the weighing of the heart. He recorded. He didn’t decide."
        art={<Thoth />}
      />
      <section className="register register--sunk" aria-label="Summary">
        <div className="wrap">
          <Answer text="Asten is a name associated with Thoth, the ibis-headed Egyptian god of writing, wisdom, the moon, and reckoning. Thoth kept the records of the gods and recorded the result when a heart was weighed against the feather of Ma’at. The writing app is named for that role: it keeps the record and marks what doesn’t balance, and the author decides." />
        </div>
      </section>

      <section className="register" aria-labelledby="n1">
        <div className="wrap split">
          <div className="name-sign" aria-hidden="true">
            <span className="sign-big">{"\u{1315D}"}</span>
            <span className="sign-caption">G26 · the ibis on a standard, which writes Thoth’s name</span>
          </div>
          <div className="prose">
            <h2 id="n1">Who was Thoth?</h2>
            <p>
              Thoth (in Egyptian, something like <em>Djehuty</em>) was one of the most important gods of ancient Egypt for more
              than two thousand years. He was usually shown with the head of an ibis, a long-legged wading bird with a curved
              beak, and sometimes as a baboon. His great cult centre was at Hermopolis, which the Egyptians called Khemenu.
            </p>
            <p>
              Thoth was credited with inventing writing, the <em>medu netjer</em>, “the god’s words”. He was the patron of
              scribes, the keeper of knowledge, and the god who measured and reckoned. A popular tradition holds that scribes
              offered him a drop of water from their pots before starting work.
            </p>
            <p>
              Egyptian gods gathered many names and epithets over three thousand years. Asten, sometimes written Astennu, is one
              associated with Thoth, and the one we chose.
            </p>
          </div>
        </div>
      </section>

      <section className="register register--sunk" aria-labelledby="n2">
        <div className="wrap split">
          <div>
            <Scales tilt={-5} left="heart" className="name-art" />
          </div>
          <div className="prose">
            <h2 id="n2">The weighing of the heart</h2>
            <p>
              In the Book of the Dead, a dead person’s heart is placed on a balance against the feather of Ma’at, the goddess of
              truth, order, and balance. Anubis tends the scales. Thoth stands beside them with his palette and records the
              result. Osiris gives the judgment.
            </p>
            <p>
              That scene is the whole idea of Asten. Your page goes in one pan; the canon you have settled goes in the other. If
              they don’t balance, Asten writes it down, in red, with the evidence. What happens next is your call.
            </p>
          </div>
        </div>
      </section>

      <section className="register register--night" aria-labelledby="n3">
        <div className="wrap split">
          <div>
            <LunarEmblem className="name-art" />
          </div>
          <div className="prose">
            <h2 id="n3">The moon and the five extra days</h2>
            <p>
              Thoth was a moon god, and the moon was how Egyptians counted time. Thoth often wears a full moon disk resting in a
              crescent on his head, the emblem drawn throughout this site.
            </p>
            <p>
              A story retold by the Greek writer Plutarch says the sky goddess Nut had been forbidden to give birth on any day of
              the year. Thoth played a game with the moon, won a share of its light, and made five new days that belonged to no
              month. On those days her children were born. A god who makes room in the calendar so a story can happen is a fair
              patron for a timeline.
            </p>
          </div>
        </div>
      </section>

      <section className="register" aria-labelledby="n4">
        <div className="wrap split">
          <div className="name-sign" aria-hidden="true">
            <PaletteMark className="name-palette" />
            <span className="sign-caption">
              {"\u{133DE}"} · Y3, the scribe’s kit: palette, water pot, and pen case
            </span>
          </div>
          <div className="prose">
            <h2 id="n4">Two inks on one palette</h2>
            <p>
              Egyptian scribes carried a wooden palette with two cakes of ink: carbon black for the text, and red ochre for
              headings, dates, verse points, and marks. Our word <em>rubric</em> comes from the Latin for red ochre, after the red
              headings of later manuscripts.
            </p>
            <p>
              The Asten logo is that palette, after the hieroglyph that means <em>scribe</em> and <em>writing</em>: one black well
              for the writer, one red well for the editor. The rule it carries runs through the product. Your words are black.
              Asten only ever writes in red.
            </p>
          </div>
        </div>
      </section>

      <Frieze kind="blocks" />

      <section className="register register--sunk" aria-labelledby="n5">
        <div className="wrap split">
          <div className="prose">
            <h2 id="n5">Names in cartouches</h2>
            <p>
              A cartouche is the oval ring Egyptians drew around royal names. The word is French for a cartridge; the story goes that
              Napoleon’s soldiers thought the shape resembled one. Asten draws your characters’ names in cartouches.
            </p>
          </div>
          <CartoucheName />
        </div>
      </section>

      <section className="register" aria-labelledby="n6">
        <div className="wrap wrap-narrow prose">
          <h2 id="n6">How we use Egyptian art</h2>
          <p>
            Egyptian art is the heritage of Egypt and of everyone who has learned from it. We try to use it with care. The
            painted plates are public-domain works from museum collections, named under each picture and on the{" "}
            <Link to="/credits">credits page</Link>; the ornaments and icons are original drawings in the flat, outlined style of
            tomb painting and papyrus, made for Asten. We don’t invent fake hieroglyphic “text”: where real signs appear, we say what they are, and the name toy above is
            labeled as a playful alphabet. And no mummy jokes.
          </p>
          <p>
            <Link to="/brand" className="link-arrow">
              Read the brand guidelines <Arrow />
            </Link>
          </p>
        </div>
      </section>

      <Related paths={["/ai", "/brand", "/about", "/features/time"]} />
      <CtaBand />
    </>
  );
}
