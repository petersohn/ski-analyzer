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
import {
  Point as OlPoint,
  MultiPolygon as OlMultiPolygon,
  LineString as OlLineString,
  MultiLineString as OlMultiLineString,
} from "ol/geom";
import { MultiPolygon, Point, LineString } from "./types/geo";
import Style from "ol/style/Style";
import Stroke from "ol/style/Stroke";
import { boundingExtent } from "ol/extent";
import { Coordinate } from "ol/coordinate";
import Icon from "ol/style/Icon";
import Fill from "ol/style/Fill";

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

type PisteStyle = {
  line: Style;
  area: Style;
};

type PisteStyles = {
  [difficulty: string]: PisteStyle;
};

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
  private readonly baseLayer = new TileLayer({
    source: new XYZ({
      url: "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    }),
  });
  private readonly liftStyle = new Style({
    stroke: new Stroke({
      color: "#333",
      width: 2,
    }),
  });
  private readonly stationStyle = new Style({
    image: new Icon({
      src: "/assets/lift/station.svg",
      size: [5, 5],
    }),
  });
  private readonly pisteStyles: PisteStyles = {
    Novice: {
      line: new Style({
        stroke: new Stroke({
          color: "#0a0",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0a04",
        }),
      }),
    },
    Easy: {
      line: new Style({
        stroke: new Stroke({
          color: "#00f",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#00f4",
        }),
      }),
    },
    Intermediate: {
      line: new Style({
        stroke: new Stroke({
          color: "#f00",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#f004",
        }),
      }),
    },
    Advanced: {
      line: new Style({
        stroke: new Stroke({
          color: "#000",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0004",
        }),
      }),
    },
    Expert: {
      line: new Style({
        stroke: new Stroke({
          color: "#000",
          width: 2,
          lineDash: [6, 4],
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#0004",
        }),
      }),
    },
    Freeride: {
      line: new Style({
        stroke: new Stroke({
          color: "#f60",
          width: 2,
          lineDash: [6, 4],
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#f604",
        }),
      }),
    },
    "": {
      line: new Style({
        stroke: new Stroke({
          color: "#888",
          width: 2,
        }),
      }),
      area: new Style({
        fill: new Fill({
          color: "#8884",
        }),
      }),
    },
  };
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

  private pointToCoordinate(p: Point): Coordinate {
    return fromLonLat([p.x, p.y], this.projection);
  }

  private createLineString(l: LineString): Coordinate[] {
    return l.map((p) => this.pointToCoordinate(p));
  }

  private createMultiPolygon(mp: MultiPolygon): OlMultiPolygon {
    return new OlMultiPolygon(
      mp.map((p) => [
        this.createLineString(p.exterior),
        ...p.interiors.map((l) => this.createLineString(l)),
      ]),
    );
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
  }

  public loadSkiArea(skiArea: SkiArea) {
    const layers = this.map.getLayers();
    layers.clear();
    layers.push(this.baseLayer);

    const lifts = skiArea.lifts
      .map((lift) => {
        const line = new Feature(
          new OlLineString(this.createLineString(lift.line.item)),
        );
        line.setStyle(this.liftStyle);

        const stations = lift.stations.map((station) => {
          const feature = new Feature(
            new OlPoint(this.pointToCoordinate(station.point)),
          );
          feature.setStyle(this.stationStyle);
          return feature;
        });

        return [line, ...stations];
      })
      .flat(1);
    const pistes = skiArea.pistes
      .map((piste) => {
        const style = this.pisteStyles[piste.difficulty];
        if (!style) {
          console.warn("Unknown difficulty", piste.difficulty);
          return [];
        }
        const areas = new Feature(this.createMultiPolygon(piste.areas));
        areas.setStyle(style.area);
        const lines = new Feature(
          new OlMultiLineString(
            piste.lines.map((line) => this.createLineString(line)),
          ),
        );
        lines.setStyle(style.line);

        return [areas, lines];
      })
      .flat(1);
    const minCoord = this.pointToCoordinate(skiArea.bounding_rect.min);
    const maxCoord = this.pointToCoordinate(skiArea.bounding_rect.max);
    layers.push(
      new VectorLayer({
        source: new VectorSource({
          features: [...lifts, ...pistes],
        }),
        minZoom: 10,
        extent: boundingExtent([minCoord, maxCoord]),
      }),
    );
    this.zoomToArea(minCoord, maxCoord);
  }
}
