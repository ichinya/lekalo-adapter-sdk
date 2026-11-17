# lekalo-adapter-sdk

Общий репозиторий необязательных SDK для авторов адаптеров [Lekalo](https://github.com/ichinya/lekalo). Отдельные пакеты для Rust, TypeScript, Go и PHP могут выпускаться независимо.

## Статус

Репозиторий подготовлен для выделения существующей реализации. Код ещё не перенесён; пакет пока не готов к установке или использованию.

- Перенос реализации: [#1](https://github.com/ichinya/lekalo-adapter-sdk/issues/1).
- Общий план: [lekalo#241](https://github.com/ichinya/lekalo/issues/241).

## Ответственность пакета

- SDK Rust/TypeScript/Go/PHP: protocol types, decoding/validation helpers, canonical bytes, serve loop и logging.
- Генератор SDK из закреплённых protocol schemas, общие vectors и compatibility records.
- Минимальные templates reference/scan/generate/verify и документация для авторов адаптеров.

## Источник переноса

`lekalo/sdk/`, `scripts/generate-adapter-sdk.mjs`, связанные SDK generators/tests и `docs/adapter-sdk.md`.

## Границы

Canonical protocol schemas и финальная host-conformance остаются авторитетом core, потребляются по точным version/digest pins. SDK не обязателен: совместимый процесс может реализовать протокол самостоятельно. Язык SDK — язык реализации адаптера, а не обязательно язык анализируемого приложения.

Версия пакета, версия provider/protocol API и версия языка или движка приложения фиксируются отдельно. Сборка и тесты должны быть автономными; источники и зависимости закрепляются точными версиями и digest. Выбор произвольного совместимого Git/local package и загрузка его зависимостей отслеживаются в [lekalo#239](https://github.com/ichinya/lekalo/issues/239).
