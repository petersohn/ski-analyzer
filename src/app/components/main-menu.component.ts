import { Component, Signal } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  NameInputDialogComponent,
  NameInputDialogData,
} from "./name-input-dialog.component";
import { MatDialog } from "@angular/material/dialog";
import { MatMenuModule } from "@angular/material/menu";
import { MatDividerModule } from "@angular/material/divider";
import { MatIconModule } from "@angular/material/icon";
import { lastValueFrom } from "rxjs";
import { ActionsService } from "@/services/actions.service";
import { MapService } from "@/services/map.service";
import { SkiAreaChooserService } from "@/services/ski-area-chooser.service";

@Component({
  selector: "main-menu",
  imports: [MatButtonModule, MatDividerModule, MatIconModule, MatMenuModule],
  templateUrl: "./main-menu.component.html",
  styleUrls: ["./main-menu.component.scss"],
})
export class MainMenuComponent {
  public loading: Signal<boolean>;
  public hasSelectableSkiArea: Signal<boolean>;

  constructor(
    private readonly dialog: MatDialog,
    public readonly mapService: MapService,
    public readonly actionsService: ActionsService,
    private readonly skiAreaChooserService: SkiAreaChooserService,
  ) {
    this.loading = this.actionsService.loading;
    this.hasSelectableSkiArea = this.skiAreaChooserService.hasChoosableSkiArea;
  }

  public async loadSkiArea(): Promise<void> {
    const path = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      this.actionsService.loadSkiArea(path);
    }
  }

  public async saveSkiArea(): Promise<void> {
    const path = await save({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      await this.actionsService.saveSkiArea(path);
    }
  }

  public async findSkiArea(): Promise<void> {
    const dialogRef = this.dialog.open<
      NameInputDialogComponent,
      NameInputDialogData
    >(NameInputDialogComponent, {
      data: {
        label: "Ski area name",
        placeholder: "regular expression, case insensitive",
      },
    });
    const result = await lastValueFrom(dialogRef.afterClosed());
    if (result) {
      this.actionsService.findSkiAreasByName(result);
    }
  }

  public async findNearbySkiAreas(): Promise<void> {
    this.actionsService.findSkiAreasByCoords(this.mapService.getScreenBounds());
  }

  public async loadCachedSkiArea(): Promise<void> {
    this.actionsService.findCachedSkiAreas();
  }

  public async loadGpx(): Promise<void> {
    const path = await open({
      filters: [{ name: "GPX", extensions: ["gpx"] }],
    });
    if (!!path) {
      this.actionsService.loadGpx(path);
    }
  }

  public async loadRoute(): Promise<void> {
    const path = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      this.actionsService.loadRoute(path);
    }
  }

  public async saveRoute(): Promise<void> {
    const path = await save({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      await this.actionsService.saveRoute(path);
    }
  }

  public async cancelAllTasks(): Promise<void> {
    await this.actionsService.cancelAllTasks();
  }
}
