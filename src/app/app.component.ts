import {
  Component,
  ElementRef,
  OnDestroy,
  OnInit,
  ViewChild,
} from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import TileLayer from "ol/layer/Tile";
import XYZ from "ol/source/XYZ";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent implements OnInit, OnDestroy {
  @ViewChild("map")
  public mapElement!: ElementRef<HTMLElement>;

  private map!: OlMap;
  private listeners: UnlistenFn[] = [];

  public async ngOnInit() {
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

    this.listeners.push(
      await listen("resized", (event) => this.onResized(event)),
    );
  }

  public ngOnDestroy() {}

  private onResized(event: any) {
    console.log(event);
  }
}
