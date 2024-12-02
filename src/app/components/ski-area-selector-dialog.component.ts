import {
  Component,
  Inject,
  ViewChild,
  ElementRef,
  HostListener,
} from "@angular/core";
import {
  MAT_DIALOG_DATA,
  MatDialogRef,
  MatDialogModule,
} from "@angular/material/dialog";
import { MatInput } from "@angular/material/input";
import { MatListModule } from "@angular/material/list";
import { MatButtonModule } from "@angular/material/button";
import { SkiAreaMetadata } from "@/types/skiArea";
import { ActionsService } from "@/services/actions.service";
import { MapService } from "@/services/map.service";

export type SkiAreaSelectorDialogData = {
  ski_areas: SkiAreaMetadata[];
};

@Component({
  selector: "ski-area-selector-dialog",
  templateUrl: "./ski-area-selector-dialog.component.html",
  styleUrl: "./ski-area-selector-dialog.component.css",
  standalone: true,
  imports: [MatInput, MatListModule, MatButtonModule, MatDialogModule],
})
export class SkiAreaSelectorDialogComponent {
  @ViewChild("name")
  public name!: ElementRef<HTMLInputElement>;

  constructor(
    @Inject(MAT_DIALOG_DATA) public data: SkiAreaSelectorDialogData,
    private readonly dialogRef: MatDialogRef<SkiAreaSelectorDialogData>,
    private readonly actionsService: ActionsService,
    private readonly mapService: MapService,
  ) {}

  @HostListener("window:keyup.escape")
  public onEscape() {
    this.cancel();
  }

  public accept(id: number) {
    this.actionsService.loadSkiAreaFromId(id);
    this.close();
  }

  public cancel() {
    this.close();
  }

  public close() {
    this.unhighlight();
    this.dialogRef.close(null);
  }

  public highlight(skiArea: SkiAreaMetadata) {
    this.mapService.addOutline(skiArea.outline);
  }

  public unhighlight() {
    this.mapService.clearOutline();
  }
}
