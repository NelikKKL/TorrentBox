# TorrentBox

Простой torrent-клиент для ПК (Windows/Linux) на Rust, вдохновлённый интерфейсом
Android-приложения **LibreTorrent**, но написанный с нуля под десктоп.

- **BitTorrent-движок:** [`librqbit`](https://github.com/ikatson/rqbit) — DHT,
  магнет-ссылки, докачка, автосохранение состояния сессии.
- **Интерфейс:** [`egui`/`eframe`](https://github.com/emilk/egui) — нативное окно
  без браузера/Electron, один бинарник.
- **Фирменный цвет:** `#A9713E` (тёплый коричневый, в тон иконке-коробке).
- **Иконка:** сгенерирована из вашего `package_1f4e6.png`.

## Возможности v1

- Добавление торрентов по magnet-ссылке и по `.torrent` файлу
- **Открытие magnet-ссылок и `.torrent` файлов из системы**: клик по
  `magnet:`-ссылке в браузере или двойной клик по `.torrent` файлу открывает
  торрент в TorrentBox. Если TorrentBox уже запущен — открывается не второе
  окно, а торрент добавляется в уже работающее (см. «Обработчик по умолчанию»
  ниже)
- Список торрентов с прогрессом, скоростью загрузки/отдачи, статусом
- Пауза / возобновление (по одному и все сразу)
- Удаление (с сохранением или удалением файлов)
- Панель деталей: размер, хэш, папка, ETA
- Светлая и тёмная тема
- Автосохранение списка торрентов между запусками
- Единая иконка на Windows и Linux (собрана из одного и того же файла —
  см. «Иконка» ниже)
- **Своя шапка окна в цвете темы** (`#A9713E`) вместо системной — работает
  одинаково на Windows и Linux
- На Windows консольное окно никогда не появляется (ни в debug, ни в release)

### Чего пока нет (в отличие от LibreTorrent)

LibreTorrent — очень зрелый проект (BitTorrent v2, RSS-автозагрузка,
потоковое воспроизведение, IP-фильтры, прокси и т.д.), сравнимый по объёму с
qBittorrent/Deluge. За один присест такое не воспроизвести один в один, поэтому
здесь — крепкий MVP с самым важным ядром функций, который дальше можно
расширять: список файлов внутри торрента с выбором, что скачивать, RSS-ленты,
лимиты скорости, список пиров. Двигок (`librqbit`) всё это умеет "под капотом" —
не хватает только UI поверх него.

## Структура проекта

```
src/
  main.rs       — точка входа, окно, иконка
  app.rs        — состояние приложения, главный layout
  engine.rs     — обёртка над librqbit::Session в отдельном потоке
  models.rs     — данные для UI (TorrentRow и т.п.)
  theme.rs      — цветовая тема (#A9713E)
  ui/           — виджеты: тулбар, список, детали, диалоги
assets/         — иконки (icon.png, torrentbox.ico)
packaging/linux — .desktop файл и иконка для .deb
.github/workflows/release.yml — CI: сборка под Windows и Ubuntu
```

## Обработчик .torrent и magnet-ссылок по умолчанию

Чтобы двойной клик по `.torrent` файлу или клик по `magnet:`-ссылке в
браузере открывал именно TorrentBox, откройте **⚙ Настройки → Файлы по
умолчанию → «Сделать TorrentBox приложением по умолчанию»**.

- **Windows:** приложение само пропишет нужные ключи в
  `HKEY_CURRENT_USER\Software\Classes` (только для вашего пользователя,
  права администратора не нужны).
- **Linux:** кнопка вызывает `xdg-mime default torrentbox.desktop …` —
  сработает, только если TorrentBox установлен из `.deb` (иначе не будет
  файла `torrentbox.desktop` в `/usr/share/applications/`). То же самое
  можно сделать вручную:
  ```bash
  xdg-mime default torrentbox.desktop application/x-bittorrent
  xdg-mime default torrentbox.desktop x-scheme-handler/magnet
  ```

Как только TorrentBox уже запущен, повторный клик по ссылке/файлу **не**
открывает новое окно — торрент добавляется в уже работающий экземпляр
(через локальный TCP-порт `127.0.0.1:58973`, наружу недоступен).

## Своя шапка окна

Системное оформление окна отключено (`with_decorations(false)` в
`src/main.rs`), а шапка нарисована вручную в `src/ui/titlebar.rs` в цвете
темы — за неё можно перетаскивать окно, двойной клик разворачивает/
восстанавливает, справа — свои кнопки свернуть/развернуть/закрыть. Работает
одинаково на Windows и Linux. Т.к. вместе с системной рамкой пропадают и
системные "ручки" для изменения размера, добавлена своя маленькая ручка в
правом нижнем углу окна.

Нюанс: на некоторых Wayland-компоузиторах (в отличие от X11) перетаскивание
безрамочных окон может вести себя чуть иначе — это ограничение протокола
Wayland, а не нашего кода. Если возникнут проблемы на конкретном
Linux-окружении, самый простой откат — убрать строку `.with_decorations(false)`
в `src/main.rs` и пересобрать: тогда вернётся системная шапка ОС.

## Иконка

Иконка на Windows (`.ico` в `.exe`, ярлыки) и на Linux (значок в меню
приложений, панели задач, `.deb`) собирается из **одного и того же файла**
`assets/icon.png` (256×256) — файлы `packaging/linux/icon-*.png` для Linux
и `assets/torrentbox.ico` для Windows перегенерированы именно из него,
поэтому выглядят идентично на обеих ОС.

## Сборка локально

Нужен установленный [Rust](https://rustup.rs/) (stable).

### Windows

```powershell
cargo build --release
# результат: target\release\torrentbox.exe
```

### Ubuntu / Debian (.deb)

```bash
sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
                  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev pkg-config libglib2.0-dev
cargo install cargo-deb
cargo deb
# результат: target/debian/torrentbox_0.1.0_amd64.deb
sudo dpkg -i target/debian/torrentbox_0.1.0_amd64.deb
```

## Автоматическая сборка через GitHub Actions

В репозитории уже настроен `.github/workflows/release.yml`. Он:

1. При каждом пуше в `main`/`master` и в pull request — просто проверяет, что
   проект собирается под Windows и Ubuntu (артефакты доступны во вкладке
   **Actions → выбранный запуск → Artifacts**).
2. При пуше git-тега вида `v0.1.0` — дополнительно создаёт **GitHub Release**
   и прикладывает к нему `torrentbox-windows-x86_64.zip` и `.deb`-пакет.

Чтобы получить готовые файлы, не собирая ничего локально:

```bash
git init && git add . && git commit -m "TorrentBox v0.1.0"
git remote add origin https://github.com/<ваш-аккаунт>/torrentbox.git
git push -u origin main
git tag v0.1.0
git push origin v0.1.0
```

После этого через несколько минут в разделе **Releases** появятся собранные
`.exe` (в zip) и `.deb`.

## О точности API

`librqbit` — активно развивающаяся библиотека. Основные вызовы
(`Session::new_with_opts`, `add_torrent`, `with_torrents`, `pause`/`unpause`,
`delete`, `ManagedTorrent::stats/name/info_hash/is_paused`) сверены по
документации на [docs.rs/librqbit](https://docs.rs/librqbit) на момент
написания (версия 8.x). `winreg` (реестр Windows), `egui::ViewportCommand`
(перетаскивание/сворачивание/изменение размера окна без системной рамки —
`StartDrag`, `Maximized`, `Minimized`, `BeginResize`) и остальной API окна —
тоже сверены по актуальной документации egui. Если после `cargo build` компилятор всё же укажет на несовпадение — почти наверняка
вышла новая версия одного из крейтов с небольшими изменениями сигнатур.
Логика общения с движком сосредоточена в `src/engine.rs`, регистрация
обработчиков файлов — в `src/register.rs`; правки, если понадобятся,
ограничатся этими файлами.

## Смена иконки/цвета

- Иконка: замените `assets/icon.png` (256×256) и пересоберите
  `assets/torrentbox.ico` любым конвертером (или Python + Pillow, как было
  сделано при создании проекта).
- Цвет: правьте константы в начале `src/theme.rs` (`ACCENT`, `ACCENT_LIGHT`,
  `ACCENT_DARK`).

## Лицензия

Код TorrentBox — MIT (см. `LICENSE`). Использует `librqbit` (Apache-2.0) и
другие open-source зависимости (см. `Cargo.toml`). Идея интерфейса
вдохновлена LibreTorrent (GPL-3.0) — сам код LibreTorrent не копировался.
