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
    public readonly actionsService: ActionsService,
  ) { }

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
      this.actionsService.findSkiArea(result);
    }
  }

  public async loadTrack(): Promise<void> {
    const path = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      this.actionsService.loadTrack(path);
    }
  }

}
