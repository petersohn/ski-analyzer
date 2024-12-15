import { Injectable, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { MapService } from "./map.service";
import { RawSkiArea, SkiAreaMetadata } from "@/types/skiArea";
import { RawTrack, Waypoint } from "@/types/track";
import { Rect } from "@/types/geo";
import { MapConfig } from "@/types/config";

@Injectable({ providedIn: "root" })
export class ActionsService {
  public loading = signal(false);
  public choosableSkiAreas = signal<SkiAreaMetadata[]>([]);

  constructor(private readonly mapService: MapService) {}

  public async loadSkiArea(path: string): Promise<void> {
    const data = JSON.parse(await invoke("load_ski_area_from_file", { path }));
    this.mapService.loadSkiArea(data as RawSkiArea, true);
  }

  public async loadSkiAreaFromId(id: number): Promise<void> {
    const data = JSON.parse(
      await this.doJob(invoke("load_ski_area_from_id", { id })),
    );
    this.mapService.loadSkiArea(data as RawSkiArea, true);
  }

  public async findSkiAreasByName(name: string): Promise<void> {
    const skiAreas = await this.doJob<SkiAreaMetadata[]>(
      invoke("find_ski_areas_by_name", { name }),
    );
    this.selectSkiAreas(skiAreas);
  }

  public async findSkiAreasByCoords(rect: Rect): Promise<void> {
    const skiAreas = await this.doJob<SkiAreaMetadata[]>(
      invoke("find_ski_areas_by_coords", { rect }),
    );
    this.selectSkiAreas(skiAreas);
  }

  public async loadGpx(path: string): Promise<void> {
    const data = JSON.parse(await this.doJob(invoke("load_gpx", { path })));
    this.mapService.loadTrack(data as RawTrack);
  }

  public async loadRoute(path: string): Promise<void> {
    const data = JSON.parse(await invoke("load_route", { path }));
    this.mapService.loadTrack(data as RawTrack);
  }

  public getSpeed(wp1: Waypoint, wp2: Waypoint): Promise<number | undefined> {
    return invoke("get_speed", { wp1, wp2 });
  }

  public getMapConfig(): Promise<MapConfig | undefined> {
    return invoke("get_map_config", {});
  }

  public saveMapConfig(config: MapConfig): Promise<void> {
    return invoke("save_map_config", { config });
  }

  private async doJob<T>(job: Promise<T>): Promise<T> {
    this.loading.set(true);
    try {
      return await job;
    } finally {
      this.loading.set(false);
    }
  }

  private async selectSkiAreas(skiAreas: SkiAreaMetadata[]) {
    if (skiAreas.length === 0) {
      return;
    }

    if (skiAreas.length === 1) {
      await this.loadSkiAreaFromId(skiAreas[0].id);
      return;
    }

    this.choosableSkiAreas.set(skiAreas);
  }
}
