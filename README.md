<div align="center">

<img align="center" width="128px" src="src-tauri/icons/icon.png" />
<h1 align="center"><b>Qopy</b></h1>

The fixed and simple clipboard manager for both Windows and Linux.

<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_x64.msi">
  <img src="./public/windows.png"> Windows (x64)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_arm64.msi">
  Windows (arm64)
</a>
<br>
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3.deb">
  <img src="./public/linux.png"> Linux (deb)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3.rpm">
  Linux (rpm)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3.AppImage">
  Linux (AppImage)
</a>
<br>
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_silicon.dmg">
  <img src="./public/apple.png"> macOS (Silicon)
</a>
•
<a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.3.3/Qopy-0.3.3_intel.dmg">
  macOS (Intel)
</a>
<br>
<br>
<sup>Nightly releases can be found <a href="https://github.com/0PandaDEV/qopy/actions/workflows/build.yml">here</a> </sup>

</div>

[discord »](https://discord.gg/invite/Y7SbYphVw9)

> \[!IMPORTANT]
>
> **Star this project**, You will receive all release notifications from GitHub without any delay \~ ⭐️

<details>
  <summary><kbd>Star History</kbd></summary>
  <a href="https://starchart.cc/0PandaDEV/Qopy">
    <picture>
      <img width="100%" src="https://starchart.cc/0PandaDEV/Qopy.svg?variant=adaptive">
    </picture>
  </a>
</details>

[![wakatime](https://wakatime.com/badge/user/018ce503-097f-4057-9599-db20b190920c/project/fe76359d-56c2-4a13-8413-55207b6ad298.svg?style=flat_square)](https://wakatime.com/badge/user/018ce503-097f-4057-9599-db20b190920c/project/fe76359d-56c2-4a13-8413-55207b6ad298)

## 📋 What is Qopy

Qopy is a fixed clipboard manager designed as a simple alternative to the standard clipboard on Windows. It aims to provide a faster, more reliable experience while providing an extensive set of features compared to its Windows counterpart.

## 🚧 Roadmap
- [x] [Setup guide](https://github.com/0PandaDEV/Qopy/blob/main/GET_STARTED.md)
- [ ] Sync Clipboard across devices https://github.com/0PandaDEV/Qopy/issues/8
- [x] Settings https://github.com/0PandaDEV/Qopy/issues/2
- [x] Metadata for copied items https://github.com/0PandaDEV/Qopy/issues/5
- [ ] Code highlighting https://github.com/0PandaDEV/Qopy/issues/7
- [ ] Streamshare integration https://github.com/0PandaDEV/Qopy/issues/4
- [ ] Content type filter https://github.com/0PandaDEV/Qopy/issues/16
- [ ] Preview for copied files https://github.com/0PandaDEV/Qopy/issues/15
- [ ] Convert files to other formats https://github.com/0PandaDEV/Qopy/issues/17
- [x] Option for custom keybind https://github.com/0PandaDEV/Qopy/issues/3
- [x] macOS Support https://github.com/0PandaDEV/Qopy/issues/13

<sup>If you have ideas for features to include, please write a feature request [here](https://github.com/0pandadev/Qopy/issues).</sup>

## 📦 Concepts

Here you can see a few concepts these might not be implemented:

![Clipboard](https://github.com/user-attachments/assets/45a44a13-6ebd-4f2d-84d2-55178e303a54)
![Settings](https://github.com/user-attachments/assets/bff5456a-f413-4e62-a43d-22c8e453aa87)


## ❤️ Donations & Support

Qopy is open-source and free to use. I appreciate donations to support ongoing development and improvements. Your contributions are voluntary and help me enhance the app for everyone.

<a href="https://buymeacoffee.com/pandadev_"><img src="https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black"/></a>

## ⌨️ Local development

You can use GitHub Codespaces for online development:

[![][codespaces-shield]][codespaces-link]

Or to get Qopy set up on your machine, you'll need to have Rust and bun installed. Then, follow these steps:

```zsh
git clone https://github.com/0pandadev/Qopy.git
cd Qopy
bun i
bun dev
```

> \[!TIP]
>
> If you are interested in contributing code, feel free to check out the [Issues](https://github.com/0pandadev/Qopy/issues) section.

## 🔨 Building for production

To build for production simply execute:

```zsh
bun build
```

> \[!NOTE]
>
> Don't worry, it will fail at the end because it can not detect a Private key, but the installer files will be generated regardless of that.
> 
> You can find them in `src-tauri/target/release/bundle`.

## 📝 License

Qopy is licensed under AGPL-3. See the [LICENSE file](./LICENCE) for more information.

[codespaces-link]: https://codespaces.new/0pandadev/Qopy
[codespaces-shield]: https://github.com/codespaces/badge.svg
