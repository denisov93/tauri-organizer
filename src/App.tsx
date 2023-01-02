import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { readText, writeText } from "@tauri-apps/api/clipboard";
import "./App.css";

type TableData = { title: string, url: string };

function App() {
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const [tableData, setTableData] = useState([
    { title: "Bloq it", url: "www.bloqit.com"},
    { title: "Bloq it", url: "www.bloqit.com"},
    { title: "Bloq it", url: "www.bloqit.com"}
  ]);

  const [links, getLinks] = useState<TableData[]>([]);
  
  async function newTableEntryHandler() {
    if (title.length > 3 && url.length > 3) {
      // setTableData(await invoke("new_table_entry", { title, url }));
      setTitle("");
      setUrl("");
    }  

  }


  // async function greet() {
  //     // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  //     setGreetMsg(await invoke("greet", { name }));
  // }

  async function getLinksHandler() {
    getLinks(await invoke("get_links"));
    console.log(links);
  }

  async function updateListOfLinks() {
    const entry: TableData = { title: "Bloq it", url: "www.bloqit.com"};
    getLinks([...links, entry]);
    await invoke("update_list_of_links", { links });
  }

  return (
    <div className="container">
      <div className="blob"/>
      <div className="blob-1"/>
      <div className="blob-2"/>
      <h1>All your links in one place</h1>

      {/* <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div> */}

      {/* <p>Click on the Tauri, Vite, and React logos to learn more.</p>

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
      </div> */}
      <div className="row top-margin">
      <input
            id="title-input"
            onChange={(e) => setTitle(e.currentTarget.value)}
            placeholder="Enter a title here."
          />
      <input
            id="link-input"
            onChange={(e) => setUrl(e.currentTarget.value)}
            placeholder="Enter an URL here..."
          />
          <button type="button" onClick={() => newTableEntryHandler()}>
            Save
          </button>
      </div>
      <div className="row top-margin">
        <table id="myTable">
          <thead>
            <tr>
              <th>Title</th>
              <th>Full URL</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tfoot>
            <tr> 
              <td colSpan={3}>
                <button type="button" onClick={() => updateListOfLinks()}>
                  Greet
                </button>
                <button type="button" onClick={() => getLinksHandler()}>
                  Get Links
                </button>
              </td>
            </tr>
          </tfoot>
          <tbody className="table-body">
            {tableData.map((value, index) => {
              return (
                <tr key={index}>
                  <td>{value.title}</td>
                  <td className="link">{value.url}</td>
                  <td>
                    <button type="button" onClick={() => {}}>
                      Greet
                    </button>
                    <button className="danger-button" type="button" onClick={() => {}}>
                      Delete
                    </button>
                  </td>
                </tr>
              )
            })}
            <tr>
              <td>Bloqit admin</td>
              <td className="link">https://stackoverflow.com/questions/17954181/scrolling-only-content-div-others-should-be-fixed</td>
              <td>
                <button type="button" onClick={() => {}}>
                  Greet
                </button>
                <button className="danger-button" type="button" onClick={() => {}}>
                  Delete
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default App;
