//! Регистрация TorrentBox как приложения для открытия `.torrent` файлов
//! и `magnet:` ссылок.
//!
//! - Windows: пишет ключи в `HKEY_CURRENT_USER\Software\Classes` — правки
//!   только для текущего пользователя, права администратора не нужны.
//! - Linux: делегирует стандартной утилите `xdg-mime` (работает, если
//!   TorrentBox установлен через .deb, т.к. использует `.desktop` файл,
//!   который deb-пакет кладёт в `/usr/share/applications/torrentbox.desktop`).

#[cfg(windows)]
pub fn register() -> anyhow::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let exe = std::env::current_exe()?;
    let exe_str = exe.display().to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // --- .torrent -----------------------------------------------------
    let (ext_key, _) = hkcu.create_subkey("Software\\Classes\\.torrent")?;
    ext_key.set_value("", &"TorrentBox.torrent")?;

    let (prog_id, _) = hkcu.create_subkey("Software\\Classes\\TorrentBox.torrent")?;
    prog_id.set_value("", &"Torrent-файл (TorrentBox)")?;

    let (icon_key, _) = hkcu.create_subkey("Software\\Classes\\TorrentBox.torrent\\DefaultIcon")?;
    icon_key.set_value("", &format!("{exe_str},0"))?;

    let (cmd_key, _) =
        hkcu.create_subkey("Software\\Classes\\TorrentBox.torrent\\shell\\open\\command")?;
    cmd_key.set_value("", &format!("\"{exe_str}\" \"%1\""))?;

    // --- magnet: --------------------------------------------------------
    let (magnet_key, _) = hkcu.create_subkey("Software\\Classes\\magnet")?;
    magnet_key.set_value("", &"URL:Magnet-ссылка")?;
    magnet_key.set_value("URL Protocol", &"")?;

    let (magnet_icon, _) = hkcu.create_subkey("Software\\Classes\\magnet\\DefaultIcon")?;
    magnet_icon.set_value("", &format!("{exe_str},0"))?;

    let (magnet_cmd, _) = hkcu.create_subkey("Software\\Classes\\magnet\\shell\\open\\command")?;
    magnet_cmd.set_value("", &format!("\"{exe_str}\" \"%1\""))?;

    Ok(())
}

#[cfg(not(windows))]
pub fn register() -> anyhow::Result<()> {
    use std::process::Command;

    let desktop_id = "torrentbox.desktop";

    let torrent = Command::new("xdg-mime")
        .args(["default", desktop_id, "application/x-bittorrent"])
        .status();
    let magnet = Command::new("xdg-mime")
        .args(["default", desktop_id, "x-scheme-handler/magnet"])
        .status();

    match (torrent, magnet) {
        (Ok(a), Ok(b)) if a.success() && b.success() => Ok(()),
        _ => anyhow::bail!(
            "Не удалось вызвать xdg-mime. Убедитесь, что TorrentBox установлен из .deb \
             (иначе .desktop файл не найден), и что пакет `xdg-utils` установлен. \
             Можно выполнить вручную:\n\
             xdg-mime default torrentbox.desktop application/x-bittorrent\n\
             xdg-mime default torrentbox.desktop x-scheme-handler/magnet"
        ),
    }
}
