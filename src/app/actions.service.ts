import { Injectable, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { MapService } from "./map.service";
import { SkiArea } from "./types/skiArea";

@Injectable({ providedIn: "root" })
export class ActionsService {
  public loading = signal(false);

  constructor(private readonly mapService: MapService) {}

  public async loadSkiArea(path: string) {
    const data = JSON.parse(await invoke("load_file", { path }));
    this.mapService.loadSkiArea(data as SkiArea);
  }

  public async findSkiArea(name: string) {
    const data = await this.doJob(invoke("find_ski_area", { name }));
    this.mapService.loadSkiArea(data as SkiArea);
  }

  private async doJob<T>(job: Promise<T>): Promise<T> {
    this.loading.set(true);
    try {
      return await job;
    } finally {
      this.loading.set(false);
    }
  }
}
