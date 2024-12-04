import { Component, Signal, HostListener } from "@angular/core";
import { MatInput } from "@angular/material/input";
import { MatListModule } from "@angular/material/list";
import { MatButtonModule } from "@angular/material/button";
import { SkiAreaMetadata } from "@/types/skiArea";
import { ActionsService } from "@/services/actions.service";
import { MapService } from "@/services/map.service";

@Component({
  selector: "ski-area-selector",
  templateUrl: "./ski-area-selector.component.html",
  styleUrl: "./ski-area-selector.component.css",
  standalone: true,
  imports: [MatInput, MatListModule, MatButtonModule],
})
export class SkiAreaSelectorComponent {
  public skiAreas: Signal<SkiAreaMetadata[]>;

  constructor(
    private readonly actionsService: ActionsService,
    private readonly mapService: MapService,
  ) {
    this.skiAreas = this.actionsService.choosableSkiAreas;
  }

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
    this.actionsService.choosableSkiAreas.set([]);
  }

  public highlight(skiArea: SkiAreaMetadata) {
    this.mapService.addOutline(skiArea.outline);
  }

  public unhighlight() {
    this.mapService.clearOutline();
  }
}
