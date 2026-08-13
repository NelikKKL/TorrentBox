fn main() {
    #[cfg(windows)]
    {
        // Встраиваем иконку и метаданные версии прямо в .exe,
        // чтобы она отображалась в проводнике, на панели задач и в ярлыке.
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/torrentbox.ico");
        res.set("ProductName", "TorrentBox");
        res.set("FileDescription", "TorrentBox — torrent-клиент");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Не удалось встроить иконку в .exe: {e}");
        }
    }
}
