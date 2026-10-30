/// <reference types="vite/client" />

// hls.js ships the light build without its own typings entry; it has the same API.
declare module "hls.js/light" {
  import Hls from "hls.js";
  export * from "hls.js";
  export default Hls;
}
