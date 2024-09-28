import { Component, AfterViewInit } from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import TileLayer from "ol/layer/Tile";
import MapBrowserEvent from "ol/MapBrowserEvent";
import XYZ from "ol/source/XYZ";
import {
  Pointer as PointerInteraction,
  defaults as defaultInteractions,
} from "ol/interaction.js";
import { toLonLat } from "ol/proj";
import Collection from "ol/Collection";
import { SkiArea } from "./types/skiArea";

class MouseMove extends PointerInteraction {
  constructor() {
    super({
      handleEvent: (evt) => this.handle(evt),
    });
  }

  private handle(event: MapBrowserEvent<any>): boolean {
    if (event.type !== "click") {
      return true;
    }
    const coord = event.map.getCoordinateFromPixel(event.pixel);
    const proj = event.map.getView().getProjection();
    const lonlat = toLonLat(coord, proj);
    console.log(lonlat);
    console.log(event.map.getView().getZoom());
    return true;
  }
}

@Component({
  selector: "map",
  standalone: true,
  imports: [],
  templateUrl: "./map.component.html",
  styleUrls: ["./map.component.css"],
})
export class MapComponent implements AfterViewInit {
  private map!: OlMap;

  public async ngAfterViewInit() {
    this.map = new OlMap({
      interactions: defaultInteractions().extend([new MouseMove()]),
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

  public loadSkiArea(skiArea: SkiArea) {
    //this.map.getLa;
  }
}
