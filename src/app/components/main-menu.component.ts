import { Component } from "@angular/core";
import { MatButton } from "@angular/material/button";
import { open } from "@tauri-apps/plugin-dialog";
import {
  NameInputDialogComponent,
  NameInputDialogData,
} from "./name-input-dialog.component";
import { MatDialog } from "@angular/material/dialog";
import { lastValueFrom } from "rxjs";
import { ActionsService } from "@/services/actions.service";
import { MapService } from "@/services/map.service";

@Component({
  selector: "main-menu",
  standalone: true,
  imports: [MatButton, NameInputDialogComponent],
  templateUrl: "./main-menu.component.html",
  styleUrls: ["./main-menu.component.css"],
})
export class MainMenuComponent {
  constructor(
    private readonly dialog: MatDialog,
    public readonly mapService: MapService,
    public readonly actionsService: ActionsService,
  ) {}

  public async loadSkiArea(): Promise<void> {
    const path = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      this.actionsService.loadSkiArea(path);
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
}
