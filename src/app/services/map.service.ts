import { Injectable, signal } from "@angular/core";
import OlMap from "ol/Map";
import OlView from "ol/View";
import Layer from "ol/layer/Layer";
import TileLayer from "ol/layer/Tile";
import VectorLayer from "ol/layer/Vector";
import MapBrowserEvent from "ol/MapBrowserEvent";
import { Style } from "ol/style";
import XYZ from "ol/source/XYZ";
import Zoom from "ol/control/Zoom";
import ScaleLine from "ol/control/ScaleLine";
import {
  Interaction,
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
import { RawTrack, Activity, TrackConverter, Waypoint } from "@/types/track";
import { MapStyleService, SelectableStyle } from "./map-style.service";
import { invoke } from "@tauri-apps/api/core";

class EventHandler extends Interaction {
  constructor(private mapService: MapService) {
    super({
      handleEvent: (evt) => this.handle(evt),
    });
  }

  private handle(event: MapBrowserEvent<any>): boolean {
    switch (event.type) {
      case "click":
        return this.handleClickEvent(event);
      case "keydown":
        return this.handleKeyEvent(event);
      default:
        return true;
    }
  }

  private handleKeyEvent(event: MapBrowserEvent<any>): boolean {
    switch (event.originalEvent.key) {
      case "ArrowLeft":
        if (event.originalEvent.shiftKey) {
          this.mapService.selectPreviousNode();
          return false;
        }
        return true;
      case "ArrowRight":
        if (event.originalEvent.shiftKey) {
          this.mapService.selectNextNode();
          return false;
        }
        return true;
      default:
        return true;
    }
  }

  private handleClickEvent(event: MapBrowserEvent<any>): boolean {
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
      const activity = feature.get("ski-analyzer-activity");
      if (activity) {
        console.log("activity", activity);
        this.mapService.selectActivity(
          activity as Activity,
          event.map.getCoordinateFromPixel(event.pixel),
        );
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

type SelectedFeature = {
  feature: Feature;
  revertStyle: Style;
};

type ActivityNode = {
  index: number;
  activity: Activity;
  coord: Coordinate;
  feature: Feature;
  waypoint: Waypoint;
  previousNode?: ActivityNode;
};

@Injectable({ providedIn: "root" })
export class MapService {
  public selectedPiste = signal<Piste | undefined>(undefined);
  public selectedLift = signal<Lift | undefined>(undefined);
  public selectedActivity = signal<Activity | undefined>(undefined);
  public selectedWaypoint = signal<Waypoint | undefined>(undefined);
  public currentWaypointSpeed = signal<number | undefined>(undefined);

  private map: OlMap | undefined;
  private readonly baseLayer = new TileLayer({
    source: new XYZ({
      url: "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
    }),
  });

  private projection: Projection | undefined;
  private targetElement: HTMLElement | undefined;

  private selectedFeatures: SelectedFeature[] = [];

  private pisteAreaFeatures: Map<Piste, Feature> = new Map();
  private pisteLineFeatures: Map<Piste, Feature> = new Map();
  private liftLineFeatures: Map<Lift, Feature> = new Map();
  private skiAreaLayer: Layer | undefined;
  private skiArea: SkiArea | undefined;

  private activityLineFeatures: Map<Activity, Feature> = new Map();
  private activityNodeFeatures: Map<Activity, ActivityNode[]> = new Map();
  private trackLayer: Layer | undefined;
  private allActivityNodes: ActivityNode[] = [];
  private selectedActivityNode: ActivityNode | undefined;

  constructor(private readonly mapStyleService: MapStyleService) {}

  public createMap(targetElement: HTMLElement) {
    if (this.isInitialized()) {
      return;
    }

    this.targetElement = targetElement;
    this.map = new OlMap({
      interactions: defaultInteractions().extend([new EventHandler(this)]),
      keyboardEventTarget: document,
      target: targetElement,
      layers: [this.baseLayer],
      view: new OlView({
        center: [0, 0],
        zoom: 2,
      }),
      controls: [new ScaleLine({ bar: true }), new Zoom()],
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
    this.unloadTrack();
    this.map!.getLayers().remove(this.skiAreaLayer);

    this.pisteAreaFeatures.clear();
    this.pisteLineFeatures.clear();
    this.liftLineFeatures.clear();
    this.skiArea = undefined;
    this.skiAreaLayer = undefined;

    this.activityLineFeatures.clear();
    this.trackLayer = undefined;
  }

  public unloadTrack(): void {
    if (!this.trackLayer) {
      return;
    }

    this.unselectFeatures();
    this.map!.getLayers().remove(this.trackLayer);

    this.activityLineFeatures.clear();
    this.trackLayer = undefined;
    this.allActivityNodes = [];
  }

  public loadSkiArea(skiArea: RawSkiArea): void {
    if (!this.isInitialized()) {
      throw new Error("Not initialized");
    }

    this.unloadSkiArea();

    try {
      const features: Feature[] = [];
      for (const lift of skiArea.lifts) {
        const line = new Feature(
          new OlLineString(this.createLineString(lift.line.item)),
        );
        line.setStyle(this.mapStyleService.liftStyle().unselected);
        line.set("ski-analyzer-lift", lift);
        features.push(line);
        this.liftLineFeatures.set(lift, line);

        for (const station of lift.stations) {
          const feature = new Feature(
            new OlPoint(this.pointToCoordinate(station.point)),
          );
          feature.setStyle(this.mapStyleService.stationStyle());
          feature.set("ski-analyzer-lift", lift);
          feature.set("ski-analyzer-lift-line", false);
          features.push(feature);
        }
      }

      const pisteStyles = this.mapStyleService.pisteStyles();

      for (const piste of skiArea.pistes) {
        const style = pisteStyles[piste.difficulty];
        if (!style) {
          console.warn("Unknown difficulty", piste.difficulty);
          continue;
        }
        const areas = new Feature(this.createMultiPolygon(piste.areas));
        areas.setStyle(style.area.unselected);
        areas.set("ski-analyzer-piste", piste);
        this.pisteAreaFeatures.set(piste, areas);

        const lines = new Feature(
          new OlMultiLineString(
            piste.lines.map((line) => this.createLineString(line)),
          ),
        );
        lines.setStyle(style.line.unselected);
        lines.set("ski-analyzer-piste", piste);
        this.pisteLineFeatures.set(piste, lines);

        features.push(areas, lines);
      }

      const minCoord = this.pointToCoordinate(skiArea.bounding_rect.min);
      const maxCoord = this.pointToCoordinate(skiArea.bounding_rect.max);

      this.skiAreaLayer = new VectorLayer({
        source: new VectorSource({
          features,
        }),
        minZoom: 10,
        extent: boundingExtent([minCoord, maxCoord]),
      });
      this.map!.getLayers().push(this.skiAreaLayer);
      this.skiArea = index_ski_area(skiArea);
      this.zoomToArea(minCoord, maxCoord);
    } catch (e) {
      this.unloadSkiArea();
      throw e;
    }
  }

  public loadTrack(trackRaw: RawTrack): void {
    if (!this.isInitialized()) {
      throw new Error("Not initialized");
    }

    if (!this.skiArea) {
      throw new Error("Ski area not loaded.");
    }

    this.unloadTrack();

    const track = new TrackConverter(this.skiArea).convertTrack(trackRaw);

    try {
      const features: Feature[] = [];

      for (const activity of track.item) {
        const styles = this.mapStyleService.routeStyles()[activity.type];

        const coords = activity.route.map((segment) =>
          segment.map((wp) => this.pointToCoordinate(wp.point)),
        );
        const lines = new Feature(new OlMultiLineString(coords));

        lines.setStyle(styles.line.unselected);
        lines.set("ski-analyzer-activity", activity);
        this.activityLineFeatures.set(activity, lines);
        features.push(lines);

        let previousNode: ActivityNode | undefined;

        const nodes: ActivityNode[] = activity.route
          .map((segment) => {
            previousNode = undefined;
            return segment.map((wp) => {
              const coord = this.pointToCoordinate(wp.point);
              const feature = new Feature(new OlPoint(coord));
              feature.setStyle(styles.node.unselected);
              features.push(feature);
              const node = {
                index: this.allActivityNodes.length,
                activity,
                coord,
                feature,
                waypoint: wp,
                previousNode,
              };
              previousNode = node;
              this.allActivityNodes.push(node);
              return node;
            });
          })
          .flat(1);
        this.activityNodeFeatures.set(activity, nodes);
      }

      this.trackLayer = new VectorLayer({
        source: new VectorSource({
          features,
        }),
        minZoom: 10,
        extent: boundingExtent([
          this.pointToCoordinate(track.bounding_rect.min),
          this.pointToCoordinate(track.bounding_rect.max),
        ]),
      });
      this.map!.getLayers().push(this.trackLayer);
    } catch (e) {
      this.unloadTrack();
      throw e;
    }
  }

  public unselectFeatures() {
    this.selectedPiste.set(undefined);
    this.selectedLift.set(undefined);
    this.selectedActivity.set(undefined);
    this.selectedActivityNode = undefined;

    for (const feature of this.selectedFeatures) {
      feature.feature.setStyle(feature.revertStyle);
    }
    this.selectedFeatures = [];
  }

  public selectPiste(piste: Piste) {
    this.unselectFeatures();

    const styles = this.mapStyleService.pisteStyles()[piste.difficulty];
    this.selectFeature(this.pisteAreaFeatures.get(piste), styles.area);
    this.selectFeature(this.pisteLineFeatures.get(piste), styles.line);

    this.selectedPiste.set(piste);
  }

  public selectLift(lift: Lift) {
    this.unselectFeatures();

    const style = this.mapStyleService.liftStyle();
    this.selectFeature(this.liftLineFeatures.get(lift), style);

    this.selectedLift.set(lift);
  }

  public selectActivity(activity: Activity, coord: Coordinate) {
    let closestNode: ActivityNode | undefined;
    const nodes = this.activityNodeFeatures.get(activity);
    if (nodes !== undefined) {
      let dist = Infinity;

      for (const node of nodes) {
        const dx = coord[0] - node.coord[0];
        const dy = coord[1] - node.coord[1];
        const d = dx * dx + dy * dy;
        if (d < dist) {
          closestNode = node;
          dist = d;
        }
      }
    }

    this.selectActivityAndNode(activity, closestNode);
  }

  public selectPreviousNode() {
    if (
      this.selectedActivityNode !== undefined &&
      this.selectedActivityNode.index > 0
    ) {
      const node = this.allActivityNodes[this.selectedActivityNode.index - 1];
      this.selectActivityAndNode(node.activity, node);
    }
  }

  public selectNextNode() {
    if (
      this.selectedActivityNode !== undefined &&
      this.selectedActivityNode.index < this.allActivityNodes.length - 1
    ) {
      const node = this.allActivityNodes[this.selectedActivityNode.index + 1];
      this.selectActivityAndNode(node.activity, node);
    }
  }

  private selectActivityAndNode(activity: Activity, node?: ActivityNode) {
    this.unselectFeatures();

    const styles = this.mapStyleService.routeStyles()[activity.type];
    this.selectFeature(this.activityLineFeatures.get(activity), styles.line);

    this.selectedActivity.set(activity);

    if (node !== undefined) {
      this.selectedActivityNode = node;
      this.selectFeature(node.feature, styles.node);
      this.selectedWaypoint.set(node.waypoint);
      if (node.previousNode !== undefined) {
        invoke("get_speed", {
          wp1: node.previousNode.waypoint,
          wp2: node.waypoint,
        }).then((speed) =>
          this.currentWaypointSpeed.set(speed as number | undefined),
        );
      }
    }
  }

  private selectFeature(feature: Feature | undefined, style: SelectableStyle) {
    if (!feature) {
      return;
    }

    feature.setStyle(style.selected);
    this.selectedFeatures.push({ feature, revertStyle: style.unselected });
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
