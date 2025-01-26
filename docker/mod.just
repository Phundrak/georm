default: start

start:
        docker compose -f compose.dev.yml up -d

stop:
        docker compose -f compose.dev.yml down

logs:
        docker compose -f compose.dev.yml logs -f

## Local Variables:
## mode: makefile
## End: