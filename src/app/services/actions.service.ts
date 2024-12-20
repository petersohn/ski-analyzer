import { Injectable, signal, computed } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { MapService } from "./map.service";
import { RawSkiArea, SkiAreaMetadata } from "@/types/skiArea";
import { RawTrack, Waypoint } from "@/types/track";
import { Rect } from "@/types/geo";
import {
  MapConfig,
  RawCachedSkiArea,
  convertCachedSkiAreas,
  CachedSkiArea,
} from "@/types/config";

@Injectable({ providedIn: "root" })
export class ActionsService {
  public loading = signal(false);

  public choosableSkiAreas = signal<SkiAreaMetadata[] | null>([]);
  public cachedSkiAreas = signal<CachedSkiArea[]>([]);
  public hasChoosableSkiArea = computed(
    () =>
      (this.choosableSkiAreas() !== null &&
        this.choosableSkiAreas()?.length !== 0) ||
      this.cachedSkiAreas().length !== 0,
  );

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

  public getActiveSkiArea(): Promise<RawSkiArea | undefined> {
    return invoke("get_active_ski_area", {});
  }

  public async getActiveRoute(): Promise<RawTrack | undefined> {
    const data = await invoke("get_active_route", {});
    return !!data ? JSON.parse(data as string) : undefined;
  }

  public async loadCachedSkiArea(uuid: string): Promise<void> {
    const skiArea = await invoke("load_cached_ski_area", { uuid });
    this.mapService.loadSkiArea(skiArea as RawSkiArea, true);
  }

  public async removeCachedSkiArea(uuid: string): Promise<void> {
    await invoke("remove_cached_ski_area", { uuid });
  }

  public clearChoosableSkiAreas(): void {
    this.choosableSkiAreas.set([]);
    this.cachedSkiAreas.set([]);
  }

  private async getAllCachedSkiAreas(): Promise<CachedSkiArea[]> {
    const skiAreas = await invoke("get_all_cached_ski_areas", {});
    return convertCachedSkiAreas(skiAreas as RawCachedSkiArea[]);
  }

  private async getCachedSkiAreasForArea(rect: Rect): Promise<CachedSkiArea[]> {
    const skiAreas = await invoke("get_cached_ski_areas_for_area", { rect });
    return convertCachedSkiAreas(skiAreas as RawCachedSkiArea[]);
  }

  private async getCachedSkiAreasByName(
    name: string,
  ): Promise<CachedSkiArea[]> {
    const skiAreas = await invoke("get_cached_ski_areas_by_name", { name });
    return convertCachedSkiAreas(skiAreas as RawCachedSkiArea[]);
  }

  private async doJob<T>(job: Promise<T>): Promise<T> {
    this.loading.set(true);
    try {
      return await job;
    } finally {
      this.loading.set(false);
    }
  }

  private async selectSkiAreas(
    cachedP: Promise<CachedSkiArea[]>,
    loadedP: Promise<SkiAreaMetadata[]>,
  ) {
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
