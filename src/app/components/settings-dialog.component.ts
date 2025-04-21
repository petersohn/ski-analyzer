import {
  Component,
  HostListener,
  Inject,
  ChangeDetectionStrategy,
} from "@angular/core";
import {
  FormGroup,
  FormControl,
  FormsModule,
  ReactiveFormsModule,
} from "@angular/forms";
import {
  MAT_DIALOG_DATA,
  MatDialogRef,
  MatDialogModule,
} from "@angular/material/dialog";
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { MatButtonToggleModule } from "@angular/material/button-toggle";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MapTileType, UiConfig } from "@/types/config";
import { MatInputModule } from "@angular/material/input";
import { MatMenuModule } from "@angular/material/menu";

export type SettingsDialogData = {
  config: UiConfig;
};

@Component({
  selector: "settings-dialog",
  templateUrl: "./settings-dialog.component.html",
  styleUrl: "./settings-dialog.component.scss",
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatButtonModule,
    MatButtonToggleModule,
    MatDialogModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatMenuModule,
    FormsModule,
    ReactiveFormsModule,
  ],
})
export class SettingsDialogComponent {
  public readonly hasCustomLocation =
    this.data.config.savedMapTiles.length !== 0;
  public readonly formGroup = new FormGroup({
    mapTileType: new FormControl<MapTileType>("OpenStreetMap"),
    mapTileUrl: new FormControl<string>(""),
  });

  constructor(
    @Inject(MAT_DIALOG_DATA) public readonly data: SettingsDialogData,
    private readonly dialogRef: MatDialogRef<SettingsDialogComponent>,
  ) {
    console.log(this.data);
    this.formGroup.controls.mapTileType.setValue(this.data.config.mapTileType);
    this.formGroup.controls.mapTileUrl.setValue(this.data.config.mapTileUrl);
  }

  @HostListener("window:keyup.enter")
  public onEnter() {
    if (this.formGroup.dirty) {
      this.save();
    }
  }

  public save() {
    this.dialogRef.close(this.formGroup.value);
  }

  public setCustomUrl(value: string) {
    this.formGroup.controls.mapTileUrl.setValue(value);
  }
}
