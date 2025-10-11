FROM --platform=$BUILDPLATFORM node:22-slim AS build
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
ENV CI=true
RUN corepack enable
COPY frontend /app
WORKDIR /app
RUN pnpm install --frozen-lockfile
RUN pnpm run build

FROM caddy:latest
COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=build /app/dist /var/www
