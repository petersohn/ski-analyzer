import { Injectable, signal } from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import Layer from "ol/layer/Layer";
import TileLayer from "ol/layer/Tile";
import VectorLayer from "ol/layer/Vector";
import MapBrowserEvent from "ol/MapBrowserEvent";
import XYZ from "ol/source/XYZ";
import {
  Pointer as PointerInteraction,
  defaults as defaultInteractions,
} from "ol/interaction.js";
import { Projection, fromLonLat } from "ol/proj";
import VectorSource from "ol/source/Vector";
import { Feature } from "ol";
import {
  Point as OlPoint,
  MultiPolygon as OlMultiPolygon,
  LineString as OlLineString,
  MultiLineString as OlMultiLineString,
} from "ol/geom";
import { boundingExtent } from "ol/extent";
import { Coordinate } from "ol/coordinate";
import { MultiPolygon, Point, LineString } from "@/types/geo";
import {
  RawSkiArea,
  SkiArea,
  Lift,
  Piste,
  index_ski_area,
} from "@/types/skiArea";
import { RawTrack, convertTrack } from "@/types/track";
import {
  liftStyle,
  liftStyleSelected,
  stationStyle,
  pisteStyles,
  routeStyles,
} from "./mapStyles";

class MouseEvent extends PointerInteraction {
  constructor(private mapService: MapService) {
    super({
      handleEvent: (evt) => this.handle(evt),
    });
  }

  private handle(event: MapBrowserEvent<any>): boolean {
    if (event.type !== "click") {
      return true;
    }
    const found = event.map.forEachFeatureAtPixel(event.pixel, (feature) => {
      const piste = feature.get("ski-analyzer-piste");
      if (piste) {
        console.log("piste", piste);
        this.mapService.selectPiste(piste as Piste);
        return true;
      }
      const lift = feature.get("ski-analyzer-lift");
      if (lift) {
        console.log("lift", lift);
        this.mapService.selectLift(lift as Lift);
        return true;
      }
      return false;
    });

    if (!found) {
      this.mapService.unselectFeatures();
    }

    return true;
  }
}

@Injectable({ providedIn: "root" })
export class MapService {
  public selectedPiste = signal<Piste | undefined>(undefined);
  public selectedLift = signal<Lift | undefined>(undefined);

  private map: OlMap | undefined;
  private readonly baseLayer = new TileLayer({
    source: new XYZ({
      url: "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    }),
  });

  private projection: Projection | undefined;
  private targetElement: HTMLElement | undefined;

  private selectedFeatures: Feature[] = [];
  private pisteAreaFeatures: Feature[] = [];
  private pisteLineFeatures: Feature[] = [];
  private liftFeatures: Feature[] = [];

  private skiAreaLayer: Layer | undefined;
  private skiArea: SkiArea | undefined;

  private trackFeatures: Feature[] = [];
  private trackLayer: Layer | undefined;

  constructor() { }

  public createMap(targetElement: HTMLElement) {
    if (this.isInitialized()) {
      return;
    }

    this.targetElement = targetElement;
    this.map = new OlMap({
      interactions: defaultInteractions().extend([new MouseEvent(this)]),
      target: targetElement,
      layers: [this.baseLayer],
      view: new OlView({
        center: [0, 0],
        zoom: 2,
      }),
    });

    this.projection = this.map.getView().getProjection();
  }

  public removeMap() {
    if (!this.isInitialized()) {
      return;
    }

    this.unloadSkiArea();

    this.map = undefined;
    this.projection = undefined;
    this.targetElement!.innerHTML = "";
    this.targetElement = undefined;
  }

  public isInitialized(): boolean {
    return !!this.map && !!this.targetElement && !!this.projection;
  }

  public unloadSkiArea(): void {
    if (!this.skiAreaLayer) {
      return;
    }

    this.unselectFeatures();
    this.map!.getLayers().remove(this.skiAreaLayer);

    this.pisteAreaFeatures = [];
    this.pisteLineFeatures = [];
    this.liftFeatures = [];
    this.skiArea = undefined;
    this.skiAreaLayer = undefined;
  }

  public unloadTrack(): void {
    if (!this.trackLayer) {
      return;
    }

    this.unselectFeatures();
    this.map!.getLayers().remove(this.trackLayer);

    this.trackFeatures = [];
    this.trackLayer = undefined;
  }

  public loadSkiArea(skiArea: RawSkiArea): void {
    if (!this.isInitialized()) {
      throw new Error("Not initialized");
    }

    this.unloadSkiArea();

    const liftFeatures = skiArea.lifts
      .map((lift) => {
        const line = new Feature(
          new OlLineString(this.createLineString(lift.line.item)),
        );
        line.setStyle(liftStyle);
        line.set("ski-analyzer-lift", lift);

        const stations = lift.stations.map((station) => {
          const feature = new Feature(
            new OlPoint(this.pointToCoordinate(station.point)),
          );
          feature.setStyle(stationStyle);
          feature.set("ski-analyzer-lift", lift);
          return feature;
        });

        return [line, ...stations];
      })
      .flat(1);

    const pisteAreaFeatures = [];
    const pisteLineFeatures = [];
    for (const piste of skiArea.pistes) {
      const style = pisteStyles[piste.difficulty].unselected;
      if (!style) {
        console.warn("Unknown difficulty", piste.difficulty);
        continue;
      }
      const areas = new Feature(this.createMultiPolygon(piste.areas));
      areas.setStyle(style.area);
      areas.set("ski-analyzer-piste", piste);
      areas.set("ski-analyzer-area", true);
      const lines = new Feature(
        new OlMultiLineString(
          piste.lines.map((line) => this.createLineString(line)),
        ),
      );
      lines.setStyle(style.line);
      lines.set("ski-analyzer-piste", piste);
      lines.set("ski-analyzer-area", false);

      pisteAreaFeatures.push(areas);
      pisteLineFeatures.push(lines);
    }

    const minCoord = this.pointToCoordinate(skiArea.bounding_rect.min);
    const maxCoord = this.pointToCoordinate(skiArea.bounding_rect.max);

    this.skiAreaLayer = new VectorLayer({
      source: new VectorSource({
        features: [
          ...liftFeatures,
          ...pisteAreaFeatures,
          ...pisteLineFeatures,
        ],
      }),
      minZoom: 10,
      extent: boundingExtent([minCoord, maxCoord]),
    });
    this.map!.getLayers().push(this.skiAreaLayer);

    this.liftFeatures = liftFeatures;
    this.pisteAreaFeatures = pisteAreaFeatures;
    this.pisteLineFeatures = pisteLineFeatures;
    this.skiArea = index_ski_area(skiArea);

    this.zoomToArea(minCoord, maxCoord);
  }

  public loadTrack(trackRaw: RawTrack): void {
    if (!this.isInitialized()) {
      throw new Error("Not initialized");
    }

    this.unloadTrack();

    const track = convertTrack(trackRaw);

    this.trackFeatures = track.item.map(activity => {
      const lines = new Feature(
        new OlMultiLineString(
          activity.route.map(segment => segment.map(wp => this.pointToCoordinate(wp.point)))
        ),
      );
      lines.setStyle(routeStyles[activity.type]);
      lines.set("ski-analyzer-activity", activity);
      return lines;
    });

    this.trackLayer = new VectorLayer({
      source: new VectorSource({
        features: this.trackFeatures,
      }),
      minZoom: 10,
      extent: boundingExtent([
        this.pointToCoordinate(track.bounding_rect.min),
        this.pointToCoordinate(track.bounding_rect.max)
      ]),
    });
    this.map!.getLayers().push(this.trackLayer);
  }

  public unselectFeatures() {
    this.selectedPiste.set(undefined);
    this.selectedLift.set(undefined);

    for (const feature of this.selectedFeatures) {
      const piste = feature.get("ski-analyzer-piste") as Piste;
      if (piste) {
        const styles = pisteStyles[piste.difficulty];
        if (styles) {
          const isArea = feature.get("ski-analyzer-area");
          feature.setStyle(
            isArea ? styles.unselected.area : styles.unselected.line,
          );
        }
      } else if (feature.get("ski-analyzer-lift")) {
        feature.setStyle(liftStyle);
      }
    }
    this.selectedFeatures = [];
  }

  public selectPiste(piste: Piste) {
    this.unselectFeatures();

    const styles = pisteStyles[piste.difficulty];

    for (const feature of this.pisteAreaFeatures) {
      if (feature.get("ski-analyzer-piste") === piste) {
        feature.setStyle(styles.selected.area);
        this.selectedFeatures.push(feature);
      }
    }

    for (const feature of this.pisteLineFeatures) {
      if (feature.get("ski-analyzer-piste") === piste) {
        feature.setStyle(styles.selected.line);
        this.selectedFeatures.push(feature);
      }
    }

    this.selectedPiste.set(piste);
  }

  public selectLift(lift: Lift) {
    this.unselectFeatures();

    for (const feature of this.liftFeatures) {
      if (feature.get("ski-analyzer-lift") === lift) {
        feature.setStyle(liftStyleSelected);
        this.selectedFeatures.push(feature);
      }
    }

    this.selectedLift.set(lift);
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
    const resolution = Math.max(
      ((max[0] - min[0]) / this.targetElement!.clientWidth) * 1.1,
      ((max[1] - min[1]) / this.targetElement!.clientHeight) * 1.1,
    );
    const view = this.map!.getView();
    view.setResolution(resolution);
    view.setCenter(center);
  }
}
