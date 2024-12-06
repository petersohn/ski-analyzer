import {
  Component,
  Signal,
  computed,
  signal,
  HostListener,
} from "@angular/core";
import { MatListModule } from "@angular/material/list";
import { MatInputModule } from "@angular/material/input";
import { MatFormFieldModule } from "@angular/material/form-field";
import { FormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { SkiAreaMetadata } from "@/types/skiArea";
import { ActionsService } from "@/services/actions.service";
import { MapService } from "@/services/map.service";
import { filterString } from "@/utils/string";

@Component({
  selector: "ski-area-selector",
  templateUrl: "./ski-area-selector.component.html",
  styleUrl: "./ski-area-selector.component.css",
  standalone: true,
  imports: [
    MatListModule,
    MatButtonModule,
    FormsModule,
    MatFormFieldModule,
    MatInputModule,
  ],
})
export class SkiAreaSelectorComponent {
  private skiAreas: Signal<SkiAreaMetadata[]>;
  public filter = signal("");
  public displayedSkiAreas = computed(() =>
    this.skiAreas().filter((a) => filterString(a.name, this.filter())),
  );

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
