import { Component, AfterViewInit } from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import TileLayer from "ol/layer/Tile";
import XYZ from "ol/source/XYZ";

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
}
