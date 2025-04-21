import { Injectable, signal } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { ActionsService } from "./actions.service";
import { UiConfig } from "@/types/config";
import {
  SettingsDialogComponent,
  SettingsDialogData,
} from "@/components/settings-dialog.component";
import { MatDialog } from "@angular/material/dialog";

@Injectable({ providedIn: "root" })
export class ConfigService {
  public readonly mapTileUrl = signal("");
  public readonly initialized = signal(false);

  private config?: UiConfig;

  constructor(
    private readonly actionsService: ActionsService,
    private readonly dialog: MatDialog,
  ) {
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

    return JSON.parse(JSON.stringify(this.config));
  }

  public async openSettings() {
    const config = this.getConfig();
    this.unAutoFill(config);
    const dialogRef = this.dialog.open<
      SettingsDialogComponent,
      SettingsDialogData,
      UiConfig
    >(SettingsDialogComponent, {
      data: { config },
    });
    const result = await lastValueFrom(dialogRef.afterClosed());
    if (result) {
      result.savedMapTiles = config.savedMapTiles;
      this.autoFill(result);
      await this.setConfig(result);
    }
  }

  private autoFill(config: UiConfig) {
    console.log(config);
    switch (config.mapTileType) {
      case "OpenStreetMap":
        config.mapTileUrl = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";
        break;
      case "Custom": {
        const url = new URL(config.mapTileUrl);
        const ignoredPath = /[{}]/;
        const path = decodeURI(url.pathname)
          .split("/")
          .filter((x) => !ignoredPath.test(x));
        const name = url.host + path.join("/");

        if (!config.savedMapTiles.some((s) => s.name === name)) {
          config.savedMapTiles.push({ name, value: config.mapTileUrl });
        }
        break;
      }
    }
  }

  private unAutoFill(config: UiConfig) {
    switch (config.mapTileType) {
      case "OpenStreetMap":
        config.mapTileUrl = "";
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
        savedMapTiles: [],
      };
      this.autoFill(config);
    } else if (!config.savedMapTiles) {
      config.savedMapTiles = [];
    }

    this.setConfig(config);
    this.initialized.set(true);
  }

  public setConfigInner(config: UiConfig) {
    this.config = JSON.parse(JSON.stringify(config));
    this.mapTileUrl.set(config.mapTileUrl);
  }
}
