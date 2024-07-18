<div align="center">
  <img width="150px" src="https://github.com/user-attachments/assets/33770922-e62e-49d7-a6ce-f82b2eec01b6"/>
</div>

# Qopy

The fixed clipboard manager. The goal is simple be a alternative to the standart clipboard manager of windows. But it also supports Linux so you can also enjoy Qopy there. 

❌ macOS will probably be not supported if you want something similar on macOS take a look at [Raycast](https://www.raycast.com/) and their clipboard history extension.
#### [⬇️ Download Qopy](https://github.com/0PandaDEV/Qopy/releases)

## 🚧 Roadmap
- [ ] Setup guide
- [ ] Option for custom keybind
- [ ] More metadata for images
- [ ] Code highlighting

## 🖼️ Preview
<img width="800px" src="https://github.com/user-attachments/assets/18e1f9e3-414c-46e2-9c51-61c6e63a06d2"/>
<img width="800px" src="https://github.com/user-attachments/assets/46ec4672-f156-4426-a2cb-3a40d00dbcd6"/>

## ❤️ Donations & Support

Qopy is an open-source project, and I rely on the support of the community to continue developing and improving the app. Although Qopy is free to use, I welcome donations from those who have found it to be a valuable app and would like to contribute to its development.

Please note that Qopy is and will always be free to use. Your donation is entirely voluntary and is not required to use the app.

<a href="https://ko-fi.com/pandadev_"><img src="https://img.shields.io/badge/Buy_Me_A_Coffee-323842?style=for-the-badge&logo=buy-me-a-coffee&logoColor=white"/></a>

Find more options by clicking the Sponsor ❤️ button on the top of this page.

## 🤝 Contributing

To start contributing to Qopy, you'll need to have Rust and pnpm installed. Then, follow these steps:

1. Clone the project using `git clone https://github.com/0PandaDEV/Qopy.git`
2. Change into the project directory: `cd Qopy`
3. Install dependencies: `pnpm i`
4. Run the development server: `pnpm dev`

For a list of how you can help me checkout the [issues section](https://github.com/0PandaDEV/Qopy/issues).

## 🛠️ Building for Production

```zsh
pnpm build
```

Don't worry, it will fail at the end because it can not detect a Private key, but the installer files will be generated regardless of that.

You can find them in `src-tauri/target/release/bundle`.

## 📝 License

Qopy is licensed under the Creative Commons Attribution-Noncommercial-Share Alike. See the [LICENSE file](./LICENCE) for more information.
