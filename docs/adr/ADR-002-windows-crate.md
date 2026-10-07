# ADR-002: Крейт `windows` (WinRT+Win32) с платформенной изоляцией

Статус: принят. Дата: 2026-10-07.

## Решение
Используем официальный `windows` crate Microsoft (версию фиксируем при скаффолдинге, актуальная мажорная линейка 0.5x) для WinRT-проекций (SMTC M8, Ocr M4, Graphics.Capture M4, CapabilityAccessManager M7) и Win32 (RegisterHotKey, AddClipboardFormatListener, SetWindowLongPtrW ex-styles, SHQueryUserNotificationState, known folders, Core Audio). Все вызовы — только внутри `#[cfg(target_os = "windows")]` подмодулей вида `modules/*/win.rs`.

## Обоснование
- `windows-sys` не содержит WinRT-проекций (зафиксировано в ТЗ §4 как отклонение от анализа).
- Платформенная изоляция позволяет компилировать и тестировать ядро (db/settings/filters/parser) на Linux/macOS в CI, снижая стоимость обратной связи.

## Последствия
- Компилляция windows-target медленная; вынос тяжёлых модулей в отдельные cargo-фичи (`win_full`) при необходимости.
- Риск обновления API-поверхности windows crate минимизируется фиксацией версии в Cargo.lock и еженедельным `cargo update --check` в PR (не блокирующим).
