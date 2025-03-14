import { Injectable, signal, computed } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { SkiAreaChooserService } from "./ski-area-chooser.service";
import { RawSkiArea, SkiAreaMetadata } from "@/types/skiArea";
import { DerivedData, RawTrack, Waypoint } from "@/types/track";
import { Rect } from "@/types/geo";
import { Error } from "@/types/error";
import {
  MapConfig,
  RawCachedSkiArea,
  convertCachedSkiAreas,
  CachedSkiArea,
  UiConfig,
} from "@/types/config";
import { TasksService } from "./tasks.service";

@Injectable({ providedIn: "root" })
export class ActionsService {
  private loadingNum = signal(0);
  public loading = computed(() => this.loadingNum() !== 0);

  constructor(
    private readonly skiAreaChooserService: SkiAreaChooserService,
    private readonly tasksService: TasksService,
  ) {}

  public async loadSkiArea(path: string): Promise<void> {
    await invoke("load_ski_area_from_file", { path });
  }

  public async saveSkiArea(path: string): Promise<void> {
    await invoke("save_current_ski_area_to_file", { path });
  }

  public async loadSkiAreaFromId(id: number): Promise<void> {
    await this.doJob(
      (async () =>
        this.tasksService.addTask(
          await invoke("load_ski_area_from_id", { id }),
        ))(),
    );
  }

  public async findSkiAreasByName(name: string): Promise<void> {
    const cached = this.getCachedSkiAreasByName(name);
    const loaded = this.doJob<SkiAreaMetadata[]>(
      (async () =>
        this.tasksService.addTask(
          await invoke("find_ski_areas_by_name", { name }),
        ))(),
    );
    await this.skiAreaChooserService.selectSkiAreas(cached, loaded);
  }

  public async findSkiAreasByCoords(rect: Rect): Promise<void> {
    const cached = this.getCachedSkiAreasForArea(rect);
    const loaded = this.doJob<SkiAreaMetadata[]>(
      (async () =>
        this.tasksService.addTask(
          await invoke("find_ski_areas_by_coords", { rect }),
        ))(),
    );
    await this.skiAreaChooserService.selectSkiAreas(cached, loaded);
  }

  public async findCachedSkiAreas(): Promise<void> {
    const cached = this.getAllCachedSkiAreas();
    const loaded = Promise.resolve(undefined);
    await this.skiAreaChooserService.selectSkiAreas(cached, loaded);
  }

  public async loadGpx(path: string): Promise<void> {
    try {
      await this.doJob(
        (async () =>
          this.tasksService.addTask(await invoke("load_gpx", { path })))(),
      );
    } catch (e) {
      const err = e as Error;
      if (err.type === "NoSkiAreaAtLocation") {
        this.skiAreaChooserService.actionOnSelect = () => {
          return this.loadGpx(path);
        };
        await this.findSkiAreasByCoords(err.details!);
      }
    }
  }

  public async loadRoute(path: string): Promise<void> {
    await invoke("load_route", { path });
  }

  public async saveRoute(path: string): Promise<void> {
    await invoke("save_current_route_to_file", { path });
  }

  public getDerivedData(wp1: Waypoint, wp2: Waypoint): Promise<DerivedData> {
    return invoke("get_derived_data", { wp1, wp2 });
  }

  public getMapConfig(): Promise<MapConfig | undefined> {
    return invoke("get_map_config", {});
  }

  public saveMapConfig(config: MapConfig): Promise<void> {
    return invoke("save_map_config", { config });
  }

  public getActiveSkiArea(): Promise<RawSkiArea | null> {
    return invoke("get_active_ski_area", {});
  }

  public async getActiveRoute(): Promise<RawTrack | undefined> {
    const data = await invoke("get_active_route", {});
    return !!data ? JSON.parse(data as string) : undefined;
  }

  public async loadCachedSkiArea(uuid: string): Promise<void> {
    await invoke("load_cached_ski_area", { uuid });
  }

  public async removeCachedSkiArea(uuid: string): Promise<void> {
    await invoke("remove_cached_ski_area", { uuid });
  }

  public async cancelAllTasks(): Promise<void> {
    await invoke("cancel_all_tasks", {});
  }

  public async getUiConfig(): Promise<UiConfig | undefined> {
    const config = (await invoke("get_ui_config", {})) as string;
    return !!config ? JSON.parse(config) : undefined;
  }

  public async setUiConfig(config: UiConfig): Promise<void> {
    await invoke("set_ui_config", { config: JSON.stringify(config) });
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
    this.loadingNum.update((n) => n + 1);
    try {
      return await job;
    } finally {
      this.loadingNum.update((n) => Math.max(n - 1, 0));
    }
  }
}
