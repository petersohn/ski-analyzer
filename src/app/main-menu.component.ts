import { Component, EventEmitter, Output } from "@angular/core";
import { MatButton } from "@angular/material/button";
import { open } from "@tauri-apps/plugin-dialog";
import {
  NameInputDialogComponent,
  NameInputDialogData,
} from "./name-input-dialog.component";
import { MatDialog } from "@angular/material/dialog";
import { lastValueFrom } from "rxjs";

@Component({
  selector: "main-menu",
  standalone: true,
  imports: [MatButton, NameInputDialogComponent],
  templateUrl: "./main-menu.component.html",
  styleUrls: ["./main-menu.component.css"],
})
export class MainMenuComponent {
  @Output()
  public onLoadSkiArea = new EventEmitter<string>();

  @Output()
  public onFindSkiArea = new EventEmitter<string>();

  constructor(private readonly dialog: MatDialog) {}

  public async loadSkiArea(): Promise<void> {
    const path = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      this.onLoadSkiArea.emit(path);
    }
  }

  public async findSkiArea(): Promise<void> {
    const dialogRef = this.dialog.open<
      NameInputDialogComponent,
      NameInputDialogData
    >(NameInputDialogComponent, {
      data: { label: "Ski area name", placeholder: "Ex. Les 3 Vallées" },
    });
    const result = await lastValueFrom(dialogRef.afterClosed());
    if (result) {
      this.onFindSkiArea.emit(result);
    }
  }
}
