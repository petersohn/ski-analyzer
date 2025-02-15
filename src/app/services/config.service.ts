import { Injectable, signal } from "@angular/core";
import { ActionsService } from "./actions.service";
import { UiConfig } from "@/types/config";

@Injectable({ providedIn: "root" })
export class ConfigService {
  public readonly mapTileUrl = signal("");
  public readonly initialized = signal(false);

  private config?: UiConfig;

  constructor(private readonly actionsService: ActionsService) {
    this.init();
  }

  public async setConfig(config: UiConfig) {
    this.setConfigInner(config);
    await this.actionsService.setUiConfig(config);
  }

  public getConfig(): UiConfig {
    if (!this.config) {
      throw new Error("Config not initialized");
    }

    return { ...this.config };
  }

  public autoFill(config: UiConfig) {
    switch (config.mapTileType) {
      case "OpenStreetMap":
        config.mapTileUrl = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";
        break;
      case "Custom":
        break;
    }
  }

  private async init() {
    let config = await this.actionsService.getUiConfig();
    if (!config) {
      config = {
        mapTileType: "OpenStreetMap",
        mapTileUrl: "",
      };
      this.autoFill(config);
    }

    this.setConfig(config);
    this.initialized.set(true);
  }

  public setConfigInner(config: UiConfig) {
    this.config = { ...config };
    this.mapTileUrl.set(config.mapTileUrl);
  }
}
