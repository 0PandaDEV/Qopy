<div align="center">

<img align="center" width="128px" src="src-tauri/icons/icon.png" />
<h1 align="center"><b>Qopy</b></h1>

The fixed, simple and reliable clipboard manager for both Windows and Linux.

<ins>**This project is not yet stable!**</ins>

<table>
  <tbody>
    <tr>
      <td>Download for</td>
      <td>
        <a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.1.1/Qopy-0.1.1.msi">
          <img src="./public/windows.png"> Windows
        </a>
      </td>
      <td>
        <a href="https://github.com/0PandaDEV/Qopy/releases/download/v0.1.1/Qopy-0.1.1.AppImage">
          <img src="./public/linux.png"> Linux
        </a>
      </td>
    </tr>
  </tbody>
</table>

<sup>Unstable Nightly releases can be found <a href="https://github.com/0PandaDEV/qopy/actions/workflows/build.yml">here</a> </sup>

</div>

> \[!IMPORTANT]
>
> **Star this project**, You will receive all release notifications from GitHub without any delay \~ ⭐️

<details>
  <summary><kbd>Star History</kbd></summary>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=0pandadev/qopy&theme=dark&type=Date">
    <img width="100%" src="https://api.star-history.com/svg?repos=0pandadev/qopy&type=Date">
  </picture>
</details>

## 📋 What is Qopy

Qopy is a fixed clipboard manager designed as a simple alternative to the standard clipboard in Windows. It aims to provide a smoother, more reliable experience. Plus, it's compatible with Linux, so you can enjoy the same great features on both operating systems.

❌ macOS will probably be not supported if you want something similar on macOS take a look at [Raycast](https://www.raycast.com/) and their clipboard history extension.

## 🚧 Roadmap
- [ ] [Setup guide](https://github.com/0PandaDEV/Qopy/blob/main/GET_STARTED.md)
- [ ] Settings https://github.com/0PandaDEV/Qopy/issues/2
- [ ] Option for custom keybind https://github.com/0PandaDEV/Qopy/issues/3
- [ ] Metadata for copied items https://github.com/0PandaDEV/Qopy/issues/5
- [ ] Code highlighting https://github.com/0PandaDEV/Qopy/issues/7
- [ ] Streamshare integration https://github.com/0PandaDEV/Qopy/issues/4
- [ ] Cross-device clipboard sharing https://github.com/0PandaDEV/Qopy/issues/8

<sup>If you have ideas for features to include, please write a feature request [here](https://github.com/0pandadev/Qopy/issues).</sup>

## 📦 Preview
<img width="800px" src="https://github.com/user-attachments/assets/18e1f9e3-414c-46e2-9c51-61c6e63a06d2"/>
<img width="800px" src="https://github.com/user-attachments/assets/46ec4672-f156-4426-a2cb-3a40d00dbcd6"/>

## ❤️ Donations & Support

Qopy is open-source and free to use. I appreciate donations to support ongoing development and improvements. Your contributions are voluntary and help me enhance the app for everyone.

<a href="https://buymeacoffee.com/pandadev_"><img src="https://img.shields.io/badge/Buy_Me_A_Coffee-FFDD00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black"/></a>

More options available via the Sponsor ❤️ button above.

## ⌨️ Local development

You can use GitHub Codespaces for online development:

[![][codespaces-shield]][codespaces-link]

Or to get Qopy set up on your machine, you'll need to have Rust and pnpm installed. Then, follow these steps:

```zsh
git clone https://github.com/0pandadev/Qopy.git
cd Qopy
pnpm i
pnpm dev
```

> \[!TIP]
>
> If you are interested in contributing code, feel free to check out our GitHub [Issues](https://github.com/0pandadev/Qopy/issues).

## 🔨 Building for production

To build for production simply execute:

```zsh
pnpm build
```

> \[!NOTE]
>
> Don't worry, it will fail at the end because it can not detect a Private key, but the installer files will be generated regardless of that.
> 
> You can find them in `src-tauri/target/release/bundle`.

## 📝 License

Qopy is licensed under the Creative Commons Attribution-Noncommercial-Share Alike. See the [LICENSE file](./LICENCE) for more information.

[codespaces-link]: https://codespaces.new/0pandadev/Qopy
[codespaces-shield]: https://github.com/codespaces/badge.svg
