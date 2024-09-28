import { Component, AfterViewInit, ElementRef, ViewChild } from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import TileLayer from "ol/layer/Tile";
import VectorLayer from "ol/layer/Vector";
import MapBrowserEvent from "ol/MapBrowserEvent";
import XYZ from "ol/source/XYZ";
import {
  Pointer as PointerInteraction,
  defaults as defaultInteractions,
} from "ol/interaction.js";
import { Projection, toLonLat, fromLonLat } from "ol/proj";
import { SkiArea } from "./types/skiArea";
import VectorSource from "ol/source/Vector";
import { Feature } from "ol";
import { LineString } from "ol/geom";
import { Point } from "./types/geo";
import Style from "ol/style/Style";
import Stroke from "ol/style/Stroke";
import { boundingExtent } from "ol/extent";
import { Coordinate } from "ol/coordinate";

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
    console.log(coord, lonlat);
    const view = event.map.getView();
    console.log(view.getZoom());
    console.log(view.getResolution());
    console.log(view.getCenter());
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
  @ViewChild("map")
  public mapElement!: ElementRef<HTMLElement>;

  private map!: OlMap;
  private baseLayer = new TileLayer({
    source: new XYZ({
      url: "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    }),
  });
  liftStyle = new Style({
    stroke: new Stroke({
      color: "#000",
      width: 2,
    }),
  });
  private projection!: Projection;

  constructor() {}

  public async ngAfterViewInit() {
    this.map = new OlMap({
      interactions: defaultInteractions().extend([new MouseMove()]),
      target: "map",
      layers: [this.baseLayer],
      view: new OlView({
        center: [0, 0],
        zoom: 2,
      }),
    });

    this.projection = this.map.getView().getProjection();
  }

  private pointToCoordinate(p: Point) {
    return fromLonLat([p.x, p.y], this.projection);
  }

  private zoomToArea(min: Coordinate, max: Coordinate) {
    const center = [(min[0] + max[0]) / 2, (min[1] + max[1]) / 2];
    const element = this.mapElement.nativeElement;
    const resolution = Math.max(
      ((max[0] - min[0]) / element.clientWidth) * 1.1,
      ((max[1] - min[1]) / element.clientHeight) * 1.1,
    );
    const view = this.map.getView();
    view.setResolution(resolution);
    view.setCenter(center);
    console.log({
      element,
      w: element.clientWidth,
      h: element.clientHeight,
      min,
      max,
      center,
      resolution,
      realCenter: view.getCenter(),
      realResolution: view.getResolution(),
    });
  }

  public loadSkiArea(skiArea: SkiArea) {
    const layers = this.map.getLayers();
    layers.clear();
    layers.push(this.baseLayer);

    const lifts = skiArea.lifts.map((lift) => {
      const feature = new Feature(
        new LineString(lift.line.item.map((p) => this.pointToCoordinate(p))),
      );
      feature.setStyle(this.liftStyle);
      return feature;
    });
    const minCoord = this.pointToCoordinate(skiArea.bounding_rect.min);
    const maxCoord = this.pointToCoordinate(skiArea.bounding_rect.max);
    layers.push(
      new VectorLayer({
        source: new VectorSource({
          features: lifts,
        }),
        minZoom: 10,
        extent: boundingExtent([minCoord, maxCoord]),
      }),
    );
    this.zoomToArea(minCoord, maxCoord);
  }
}
