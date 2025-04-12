import { ChangeDetectionStrategy, Component, computed } from "@angular/core";
import { MapService } from "@/services/map.service";

@Component({
  selector: "debug-info",
  imports: [],
  templateUrl: "./debug-info.component.html",
  styleUrls: ["./debug-info.component.scss"],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DebugInfoComponent {
  public readonly mouseCoordinate = computed(() => {
    const p = this.mapService.mouseCoordinate();
    return p ? `${p.x}, ${p.y}` : "";
  });

  constructor(private readonly mapService: MapService) {}
}
