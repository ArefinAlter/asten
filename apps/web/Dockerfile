FROM node:24-bookworm-slim AS builder
WORKDIR /workspace
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
COPY apps/web ./apps/web
COPY apps/desktop/package.json ./apps/desktop/package.json
COPY packages/contracts ./packages/contracts
RUN pnpm install --frozen-lockfile
RUN pnpm --filter @asten/web build

FROM nginx:1.29-alpine
COPY apps/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /workspace/apps/web/dist /usr/share/nginx/html
EXPOSE 80
