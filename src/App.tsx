import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/tauri";
import { readText, writeText } from "@tauri-apps/api/clipboard";
import { message } from '@tauri-apps/api/dialog';
import "./App.css";

type TableData = { id: string, title: string, url: string };

function App() {
  const [title, setTitle] = useState("");
  const [url, setUrl] = useState("");
  const [links, getLinks] = useState<TableData[]>([]);
  


  async function getLinksLocation() {
      // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
      await message(await invoke("get_links_location"), 'Links');
  }

  const openInNewTab = (url:string) => {
    window.open(url, "_blank", "noreferrer");
  };

  async function getLinksHandler() {
    getLinks(await invoke("get_links"));
  }

  async function updateListOfLinks() {
    if (title.length < 3 && url.length < 3) {
      return await message('Tauri is awesome', 'Tauri');
    }
    if (!url.includes("http")) {
      return await message('Please enter a valid URL', 'Tauri');
    }
    const entry: TableData = { id:`links-${title}`, title, url };
    await invoke("update_list_of_links", { links : [...links, entry] });
    setTitle("");
    setUrl("");
    await getLinksHandler();
  }

  async function deleteLinkHandler(entry: TableData) {
    const filteredLinks = links.filter((link) => link.url !== entry.url && link.title !== entry.title);
    await invoke("update_list_of_links", { links : filteredLinks });
    await getLinksHandler();
  }

  useEffect(() => {
    getLinksHandler();
  }, []);

  return (
    <div className="container">
      <div className="blob"/>
      <div className="blob-1"/>
      <div className="blob-2"/>
      <h1>All your links in one place</h1>

      <div className="row top-margin">
      <input
            id="title-input"
            value={title}
            onChange={(e) => setTitle(e.currentTarget.value)}
            placeholder="Enter a title here."
          />
      <input
            id="link-input"
            value={url}
            onChange={(e) => setUrl(e.currentTarget.value)}
            placeholder="Enter an URL here..."
          />
          <button className="danger-button" type="button" onClick={() => updateListOfLinks()}>
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
          <tbody className="table-body">
            {links.map((value, index) => {
              return (
                <tr key={index}>
                  <td>{value.title}</td>
                  <td className="link">{value.url}</td>
                  <td>
                    <button className="danger-button" type="button" role="link" onClick={() => openInNewTab(value.url)}>
                      Go To Url
                    </button>
                    <button type="button" onClick={() => {deleteLinkHandler(value)}}>
                      Delete
                    </button>
                  </td>
                </tr>
              )
            })}
          </tbody>
          <tfoot>
            <tr> 
              <td colSpan={3}>
                <button type="button" onClick={() => getLinksLocation()}>
                  Get Links Location
                </button>
                <button type="button" onClick={() => {}}>
                  Get Links
                </button>
              </td>
            </tr>
          </tfoot>
        </table>
      </div>
    </div>
  );
}

export default App;
