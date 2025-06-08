import { invoke } from "@tauri-apps/api/core";
import { HistoryItem } from "../types/types";

const { $history } = useNuxtApp();
const { hideApp } = useAppControl();

export function useActions() {
  const isProcessing = ref(false);

  const handleAction = async (action: string, item?: HistoryItem) => {
    if (!item && action !== "settings" && action !== "delete-all") return;

    isProcessing.value = true;

    try {
      switch (action) {
        case "paste-to-app":
          await pasteToCurrentApp(item);
          break;
        case "copy":
          // await copyToClipboard(item);
          break;
        case "delete":
          await deleteEntry(item);
          break;
        case "delete-all":
          // await deleteAllEntries();
          break;
        case "settings":
          openSettings();
          break;
        case "paste-plain":
          // await pasteAsPlainText(item);
          break;
        case "edit-text":
          // openTextEditor(item);
          break;
        case "rotate-image":
          // await rotateImage(item);
          break;
        case "resize-image":
          // openImageResizer(item);
          break;
        case "compress-image":
          // await compressImage(item);
          break;
        case "open-file":
          // await openFile(item);
          break;
        case "compress-file":
          // await compressFile(item);
          break;
        case "open-link":
          // await openInBrowser(item);
          break;
        case "copy-hex":
          // await copyColorFormat(item, "hex");
          break;
        case "copy-rgba":
          // await copyColorFormat(item, "rgba");
          break;
        case "copy-hsla":
          // await copyColorFormat(item, "hsla");
          break;
        default:
          console.warn(`Action ${action} not implemented`);
      }
    } catch (error) {
      console.error(`Error executing action ${action}:`, error);
    } finally {
      isProcessing.value = false;
    }
  };

  const pasteToCurrentApp = async (item?: HistoryItem) => {
    if (!item) return;

    let content = item.content;
    let contentType: string = item.content_type;
    if (contentType === "image") {
      try {
        content = await $history.readImage({ filename: content });
      } catch (error) {
        console.error("Error reading image file:", error);
        return;
      }
    }
    await hideApp();
    await $history.writeAndPaste({ content, contentType });
  };

  // const copyToClipboard = async (item?: HistoryItem) => {
  //   if (!item) return;

  //   try {
  //     switch (item.content_type) {
  //       case ContentType.Text:
  //       case ContentType.Link:
  //       case ContentType.Code:
  //         await writeText(item.content);
  //         break;
  //       case ContentType.Image:
  //         await invoke("copy_image_to_clipboard", { path: item.file_path });
  //         break;
  //       case ContentType.File:
  //         await invoke("copy_file_reference", { path: item.file_path });
  //         break;
  //       case ContentType.Color:
  //         await writeText(item.content);
  //         break;
  //       default:
  //         console.warn(`Copying type ${item.content_type} not implemented`);
  //     }
  //   } catch (error) {
  //     console.error("Failed to copy to clipboard:", error);
  //   }
  // };

  const deleteEntry = async (item?: HistoryItem) => {
    if (!item) return;
    try {
      await invoke("delete_history_item", { id: item.id });
    } catch (error) {
      console.error("Failed to delete entry:", error);
    }
  };

  // const deleteAllEntries = async () => {
  //   try {
  //     await invoke('delete_all_history');
  //   } catch (error) {
  //     console.error('Failed to delete all entries:', error);
  //   }
  // };

  const openSettings = () => {
    navigateTo("/settings");
  };

  // const pasteAsPlainText = async (item?: HistoryItem) => {
  //   if (!item) return;
  //   try {
  //     await invoke('paste_as_plain_text', { content: item.content });
  //   } catch (error) {
  //     console.error('Failed to paste as plain text:', error);
  //   }
  // };

  // const openTextEditor = (item?: HistoryItem) => {
  //   if (!item) return;
  //   // Implement logic to open text editor with the content
  //   // This might use Nuxt router or a modal based on your app architecture
  // };

  // const rotateImage = async (item?: HistoryItem) => {
  //   if (!item || item.content_type !== ContentType.Image) return;
  //   try {
  //     await invoke('rotate_image', { path: item.file_path });
  //   } catch (error) {
  //     console.error('Failed to rotate image:', error);
  //   }
  // };

  // const openImageResizer = (item?: HistoryItem) => {
  //   if (!item || item.content_type !== ContentType.Image) return;
  //   // Implement logic to open image resizer UI for this image
  // };

  // const compressImage = async (item?: HistoryItem) => {
  //   if (!item || item.content_type !== ContentType.Image) return;
  //   try {
  //     await invoke('compress_image', { path: item.file_path });
  //   } catch (error) {
  //     console.error('Failed to compress image:', error);
  //   }
  // };

  // const openFile = async (item?: HistoryItem) => {
  //   if (!item || item.content_type !== ContentType.File) return;
  //   try {
  //     await invoke('open_file', { path: item.file_path });
  //   } catch (error) {
  //     console.error('Failed to open file:', error);
  //   }
  // };

  // const compressFile = async (item?: HistoryItem) => {
  //   if (!item || item.content_type !== ContentType.File) return;
  //   try {
  //     await invoke('compress_file', { path: item.file_path });
  //   } catch (error) {
  //     console.error('Failed to compress file:', error);
  //   }
  // };

  // const openInBrowser = async (item?: HistoryItem) => {
  //   if (!item || item.content_type !== ContentType.Link) return;
  //   try {
  //     await invoke('open_url', { url: item.content });
  //   } catch (error) {
  //     console.error('Failed to open URL in browser:', error);
  //   }
  // };

  // const copyColorFormat = async (item?: HistoryItem, format: 'hex' | 'rgba' | 'hsla' = 'hex') => {
  //   if (!item || item.content_type !== ContentType.Color) return;
  //   try {
  //     const formattedColor = await invoke('get_color_format', {
  //       color: item.content,
  //       format
  //     });
  //     await writeText(formattedColor as string);
  //   } catch (error) {
  //     console.error(`Failed to copy color as ${format}:`, error);
  //   }
  // };

  return {
    handleAction,
    isProcessing,
  };
}
