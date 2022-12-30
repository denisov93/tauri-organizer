import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { readText, writeText } from "@tauri-apps/api/clipboard";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [links, getLinks] = useState([]);
  const [clipboard, setClipboard] = useState("") ;
  const [clipResult, setClipResult] = useState("") ;

  const [arrayOfclipboard, setArrayOfclipboard] = useState(['']) ;
  
  useEffect(() => {
    (async () => {
      // const clipboardText = await readText() as string;
      // setClipboard(clipboardText);
      // setClipboardTray(); 
      // const lastElem = arrayOfclipboard.at(-1) as string;
      // if (lastElem !== clipboard) {
      //   setArrayOfclipboard(a=>[...a,clipboard]);
      //   console.log(arrayOfclipboard);
      // }
    })();
  });
  

  async function setClipboardTray() {
    setClipResult(await invoke("set_clipboard_tray", { clipboard }));
  }

  async function greet() {
      // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
      setGreetMsg(await invoke("greet", { name }));
  }

  async function getLinksHandler() {
    getLinks(await invoke("get_links"));
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <div className="row">
        <div>
          <input
            id="greet-input"
            onChange={(e) => setName(e.currentTarget.value)}
            placeholder="Enter a name..."
          />
          <button type="button" onClick={() => greet()}>
            Greet
          </button>
          <button type="button" onClick={() => getLinksHandler()}>
            Get Links
          </button>
        </div>
      </div>
      <p>{greetMsg}</p>
      <p>{links}</p>
      <p>{clipboard}</p>
      <hr/>
      <p>{arrayOfclipboard}</p>
    </div>
  );
}

export default App;
