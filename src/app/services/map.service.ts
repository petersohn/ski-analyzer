import { effect, Injectable, signal } from "@angular/core";
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
import { Projection, fromLonLat, toLonLat } from "ol/proj";
import VectorSource from "ol/source/Vector";
import { Feature } from "ol";
import {
  Point as OlPoint,
  MultiPolygon as OlMultiPolygon,
  Polygon as OlPolygon,
  LineString as OlLineString,
  MultiLineString as OlMultiLineString,
} from "ol/geom";
import { Coordinate } from "ol/coordinate";
import { MultiPolygon, Point, LineString, Rect, Polygon } from "@/types/geo";
import { SkiArea, Lift, Piste } from "@/types/skiArea";
import { Activity, DerivedData, Track, Waypoint } from "@/types/track";
import { MapStyleService, SelectableStyle } from "./map-style.service";
import { ActionsService } from "./actions.service";
import { invoke } from "@tauri-apps/api/core";
import dayjs from "dayjs";
import duration from "dayjs/plugin/duration";
import { MapConfig } from "@/types/config";
import { ConfigService } from "./config.service";
import { Pixel } from "ol/pixel";

dayjs.extend(duration);

class EventHandler extends Interaction {
  constructor(private mapService: MapService) {
    super({
      handleEvent: (evt) => this.handle(evt),
    });
  }

  private handle(event: MapBrowserEvent<any>): boolean {
    //console.log(event.type);
    switch (event.type) {
      case "click":
        return this.handleClickEvent(event);
      case "keydown":
        return this.handleKeyEvent(event);
      case "pointermove":
        return this.handlePointerMoveEvent(event);
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

  private selectFeatureAt(map: OlMap, pixel: Pixel): boolean {
    return (
      map.forEachFeatureAtPixel(pixel, (feature) => {
        const piste = feature.get("ski-analyzer-piste");
        if (piste) {
          this.mapService.selectPiste(piste as Piste);
          return true;
        }
        const lift = feature.get("ski-analyzer-lift");
        if (lift) {
          this.mapService.selectLift(lift as Lift);
          return true;
        }
        const activity = feature.get("ski-analyzer-activity");
        if (activity) {
          this.mapService.selectActivity(
            activity as Activity,
            map.getCoordinateFromPixel(pixel),
          );
          return true;
        }
        return false;
      }) ?? false
    );
  }

  private handleClickEvent(event: MapBrowserEvent<any>): boolean {
    const [x, y] = event.pixel;
    for (const [dx, dy] of [
      [0, 0],
      [0, -1],
      [1, 0],
      [0, 1],
      [-1, 0],
      [-1, -1],
      [1, -1],
      [1, 1],
      [-1, 1],
      [0, -2],
      [2, 0],
      [0, 2],
      [-2, 0],
    ]) {
      if (this.selectFeatureAt(event.map, [x + dx, y + dy])) {
        return true;
      }
    }

    this.mapService.unselectFeatures();

    return true;
  }

  private handlePointerMoveEvent(event: MapBrowserEvent<any>): boolean {
    const coord = event.map.getCoordinateFromPixel(event.pixel);
    this.mapService.mouseCoordinate.set(
      this.mapService.coordinateToPoint(coord),
    );
    return false;
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
  public readonly skiArea = signal<SkiArea | undefined>(undefined);
  public readonly selectedPiste = signal<Piste | undefined>(undefined);
  public readonly selectedLift = signal<Lift | undefined>(undefined);
  public readonly selectedActivity = signal<Activity | undefined>(undefined);
  public readonly selectedWaypoint = signal<Waypoint | undefined>(undefined);
  public readonly currentWaypointSpeed = signal<number | null>(null);
  public readonly currentWaypointInclination = signal<number | null>(null);
  public readonly currentWaypointClosestLift = signal<{
    lift: Lift;
    distance: number;
  } | null>(null);
  public readonly isInitialized = signal(false);
  public readonly mouseCoordinate = signal<Point | undefined>(undefined);

  public readonly mapConfig = signal<MapConfig | undefined>(undefined);

  private map: OlMap | undefined;
  private baseLayer: Layer | undefined;

  private projection: Projection | undefined;
  private targetElement: HTMLElement | undefined;

  private selectedFeatures: SelectedFeature[] = [];

  private pisteAreaFeatures: Map<Piste, Feature> = new Map();
  private pisteLineFeatures: Map<Piste, Feature> = new Map();
  private liftLineFeatures: Map<Lift, Feature> = new Map();
  private skiAreaLayer: Layer | undefined;

  private activityLineFeatures: Map<Activity, Feature> = new Map();
  private activityNodeFeatures: Map<Activity, ActivityNode[]> = new Map();
  private trackLayer: Layer | undefined;
  private allActivityNodes: ActivityNode[] = [];
  private selectedActivityNode: ActivityNode | undefined;

  private outlineLayer: Layer | undefined;

  constructor(
    private readonly mapStyleService: MapStyleService,
    private readonly actionsService: ActionsService,
    private readonly configService: ConfigService,
  ) {
    effect(() => {
      const url = this.configService.mapTileUrl();
      if (!!this.map && !!this.baseLayer) {
        this.map.removeLayer(this.baseLayer);
      }

      if (!url) {
        return;
      }

      this.baseLayer = new TileLayer({
        source: new XYZ({
          url,
        }),
      });

      if (!!this.map) {
        this.map.getLayers().insertAt(0, this.baseLayer);
      }
    });
  }

  public async createMap(targetElement: HTMLElement) {
    if (this.isInitialized()) {
      return;
    }

    this.targetElement = targetElement;
    this.map = new OlMap({
      interactions: defaultInteractions().extend([new EventHandler(this)]),
      keyboardEventTarget: document,
      target: targetElement,
      layers: !!this.baseLayer ? [this.baseLayer] : [],
      view: new OlView({
        center: [0, 0],
        zoom: 2,
      }),
      controls: [new ScaleLine({ bar: true }), new Zoom()],
    });

    this.projection = this.map.getView().getProjection();

    this.isInitialized.set(true);

    this.map.getView().on("change", () => {
      this.saveMapConfig();
    });
  }

  private saveMapConfig() {
    const view = this.map!.getView();
    const center = view.getCenter();
    const zoom = view.getZoom();
    if (center !== undefined && zoom !== undefined) {
      this.mapConfig.set({
        center: this.coordinateToPoint(center),
        zoom,
      });
    }
  }

  public removeMap() {
    if (!this.isInitialized()) {
      return;
    }

    this.unloadSkiArea();

    this.isInitialized.set(false);
    this.map = undefined;
    this.projection = undefined;
    this.targetElement!.innerHTML = "";
    this.targetElement = undefined;
    this.outlineLayer = undefined;
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
    this.skiArea.set(undefined);
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

  public loadSkiArea(skiArea: SkiArea, zoom: boolean): void {
    if (!this.isInitialized()) {
      throw new Error("Not initialized");
    }

    this.unloadSkiArea();

    try {
      const features: Feature[] = [];
      for (const lift of skiArea.lifts.values()) {
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

      for (const piste of skiArea.pistes.values()) {
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
      });
      this.map!.getLayers().push(this.skiAreaLayer);
      this.skiArea.set(skiArea);

      if (zoom) {
        this.zoomToArea(minCoord, maxCoord);
      }
    } catch (e) {
      this.unloadSkiArea();
      throw e;
    }
  }

  public loadTrack(track: Track): void {
    if (!this.isInitialized()) {
      throw new Error("Not initialized");
    }

    if (!this.skiArea()) {
      throw new Error("Ski area not loaded.");
    }

    this.unloadTrack();

    try {
      const features: Feature[] = [];

      let previousNode: ActivityNode | undefined;

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

        const nodes: ActivityNode[] = activity.route
          .map((segment) => {
            if (previousNode !== undefined) {
              const timeDiff =
                segment.length > 0 &&
                !!segment[0].time &&
                !!previousNode.waypoint.time
                  ? dayjs.duration(
                      segment[0].time.diff(previousNode.waypoint.time),
                    )
                  : undefined;
              if (
                timeDiff !== undefined &&
                timeDiff > dayjs.duration(0) &&
                timeDiff < dayjs.duration({ minutes: 1 })
              ) {
                const feature = new Feature(
                  new OlLineString([
                    this.pointToCoordinate(previousNode.waypoint.point),
                    this.pointToCoordinate(segment[0].point),
                  ]),
                );
                feature.setStyle(this.mapStyleService.connectorStyle());
                features.push(feature);
              } else {
                previousNode = undefined;
              }
            }
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
    this.selectedWaypoint.set(undefined);
    this.currentWaypointSpeed.set(null);
    this.currentWaypointInclination.set(null);
    this.currentWaypointClosestLift.set(null);

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

  public getScreenBounds(): Rect {
    const min = this.coordinateToPoint(
      this.map?.getCoordinateFromPixel([0, this.targetElement!.clientHeight])!,
    );
    const max = this.coordinateToPoint(
      this.map?.getCoordinateFromPixel([this.targetElement!.clientWidth, 0])!,
    );

    return { min, max };
  }

  public clearOutline() {
    if (this.outlineLayer !== undefined) {
      this.map!.removeLayer(this.outlineLayer);
    }
  }

  public addOutline(outline: Polygon) {
    this.clearOutline();
    const outer = outline.exterior.map((p) => this.pointToCoordinate(p));
    const inners = outline.interiors.map((l) =>
      l.map((p) => this.pointToCoordinate(p)),
    );
    const feature = new Feature(new OlPolygon([outer, ...inners]));
    feature.setStyle(this.mapStyleService.outlineStyle());
    this.outlineLayer = new VectorLayer({
      source: new VectorSource({
        features: [feature],
      }),
    });

    this.map!.addLayer(this.outlineLayer);
  }

  public setMapConfig(config: MapConfig) {
    if (!this.isInitialized()) {
      return;
    }

    const view = this.map!.getView();
    view.setCenter(this.pointToCoordinate(config.center));
    view.setZoom(config.zoom);
  }

  private selectActivityAndNode(activity: Activity, node?: ActivityNode) {
    this.unselectFeatures();

    const styles = this.mapStyleService.routeStyles()[activity.type];
    this.selectFeature(this.activityLineFeatures.get(activity), styles.line);

    this.selectedActivity.set(activity);

    if (node === undefined) {
      return;
    }

    this.selectedActivityNode = node;
    this.selectFeature(node.feature, styles.node);
    this.selectedWaypoint.set(node.waypoint);
    if (node.previousNode !== undefined) {
      this.actionsService
        .getDerivedData(node.previousNode.waypoint, node.waypoint)
        .then((derivedData: DerivedData) => {
          this.currentWaypointSpeed.set(derivedData.speed);
          this.currentWaypointInclination.set(derivedData.inclination);
        });
    }

    invoke("get_closest_lift", {
      p: node.waypoint.point,
      limit: 100.0,
    }).then((data) => {
      if (!data) {
        this.currentWaypointClosestLift.set(null);
      } else {
        let { lift_id, distance } = data as {
          lift_id: string;
          distance: number;
        };
        this.currentWaypointClosestLift.set({
          lift: this.skiArea()!.lifts.get(lift_id)!,
          distance,
        });
      }
    });

    this.ensureWithinView(this.pointToCoordinate(node.waypoint.point));
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

  public coordinateToPoint(c: Coordinate): Point {
    const lonlat = toLonLat(c, this.projection);
    return { x: lonlat[0], y: lonlat[1] };
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
    this.saveMapConfig();
  }

  private ensureWithinView(coord: Coordinate) {
    const view = this.map!.getView();
    const center = view.getCenter();
    if (!center) {
      view.setCenter(coord);
      return;
    }
    const [cx, cy] = center;
    const resolution = view.getResolution();
    if (!resolution) {
      return;
    }
    const dx = this.targetElement!.clientWidth * 0.4 * resolution;
    const minx = cx - dx;
    const maxx = cx + dx;
    const dy = this.targetElement!.clientHeight * 0.4 * resolution;
    const miny = cy - dy;
    const maxy = cy + dy;

    let [x, y] = coord;
    if (x < minx || x > maxx || y < miny || y > maxy) {
      view.setCenter(coord);
    }
  }
}
