// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, {Cursive} from './pkg/web.js';

// async function run() {
//     try {
//         // First up we need to actually load the wasm file, so we use the
//         // default export to inform it where the wasm file is located on the
//         // server, and then we wait on the returned promise to wait for the
//         // wasm to be loaded.
//         // It may look like this: `await init('./pkg/without_a_bundler_bg.wasm');`,
//         // but there is also a handy default inside `init` function, which uses
//         // `import.meta` to locate the wasm file relatively to js file
//         //
//         // Note that instead of a string here you can also pass in an instance
//         // of `WebAssembly.Module` which allows you to compile your own module.
//         // Also note that the promise, when resolved, yields the wasm module's
//         // exports which is the same as importing the `*_bg` module in other
//         // modes
        
//     } catch(e) {
//         console.error(e);
//     }
// }

await init();

const canvas = document.getElementById("cursive-wasm-canvas");
canvas.style.display = "block";
canvas.setAttribute("width", "1000");
canvas.setAttribute("height", "1000");
const ctx = canvas.getContext("2d");
ctx.fillStyle = "green";
// Add a rectangle at (10, 10) with size 100x100 pixels
ctx.fillRect(0, 0, 1000, 1000);
console.log("canvas is loaded", canvas !== null, ctx !== null);
Cursive.retris_with_canvas(canvas);
