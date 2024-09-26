import { Component, OnDestroy, AfterViewInit } from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import TileLayer from "ol/layer/Tile";
import XYZ from "ol/source/XYZ";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { MainMenuComponent } from "./main-menu.component";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [MainMenuComponent],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent implements AfterViewInit, OnDestroy {
  private map!: OlMap;
  private listeners: UnlistenFn[] = [];

  public async ngAfterViewInit() {
    this.map = new OlMap({
      target: "map",
      layers: [
        new TileLayer({
          source: new XYZ({
            url: "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
          }),
        }),
      ],
      view: new OlView({
        center: [0, 0],
        zoom: 2,
      }),
    });
  }

  public ngOnDestroy() {
    this.listeners.forEach((f) => f());
  }

  public async loadSkiArea(path: string) {
    const data = JSON.parse(await invoke("load_file", { path }));
    console.log(data);
  }
}
