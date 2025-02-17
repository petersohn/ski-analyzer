import { Component, Inject, ChangeDetectionStrategy } from "@angular/core";
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
import { MatButtonToggleModule } from "@angular/material/button-toggle";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MapTileType, UiConfig } from "@/types/config";
import { MatInputModule } from "@angular/material/input";

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
    MatInputModule,
    FormsModule,
    ReactiveFormsModule,
  ],
})
export class SettingsDialogComponent {
  public readonly formGroup = new FormGroup({
    mapTileType: new FormControl<MapTileType>("OpenStreetMap"),
    mapTileUrl: new FormControl<string>(""),
  });

  constructor(
    @Inject(MAT_DIALOG_DATA) private data: SettingsDialogData,
    private readonly dialogRef: MatDialogRef<SettingsDialogComponent>,
  ) {
    this.formGroup.controls.mapTileType.setValue(this.data.config.mapTileType);
    this.formGroup.controls.mapTileUrl.setValue(this.data.config.mapTileUrl);
  }

  public save() {
    this.dialogRef.close(this.formGroup.value);
  }
}
