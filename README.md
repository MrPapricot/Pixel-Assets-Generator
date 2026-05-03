# Запуск

## Если Docker установлен

``` bash
cd Docker
docker compose up
```

Для полной персборки

``` bash
cd Docker 
docker compose up --build
```

## Без использования Docker (требует cargo и база данных с необходимыми таблицами")

``` bash
cd AuthorizationService && cargo run && cd ../
cd CommandProjectBackend && cargo run && cd ../
```

Необходимые таблицы:

``` sql
users (
    uuid uuid PRIMARY KEY DEFAULT uuidv7(),
    email text NOT NULL UNIQUE,
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
)
```
