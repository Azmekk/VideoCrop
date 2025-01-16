import { check } from "@tauri-apps/plugin-updater";

export async function updateApp(setUpdateAvailable: (updateAvailable: boolean) => void, setUpdatingApp: (updatingApp: boolean) => void) {
  try {
    setUpdatingApp(true);
    const update = await check();
    if (update) {
      console.log(`found update ${update.version} from ${update.date} with notes ${update.body}`);
      let downloaded = 0;
      let contentLength = 0;
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0;
            console.log(`started downloading ${event.data.contentLength} bytes`);
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            console.log(`downloaded ${downloaded} from ${contentLength}`);
            break;
          case "Finished":
            console.log("download finished");
            break;
        }
      });

      console.log("update installed");
    }
  } catch (error) {
    alert(`Error while updating: ${error}`);
  } finally {
    setUpdatingApp(false);
    setUpdateAvailable(false);
  }
}
