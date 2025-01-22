import { Component, input, output, computed } from "@angular/core";
import { MatButtonModule } from "@angular/material/button";
import { MatListModule } from "@angular/material/list";
import { MatIconModule } from "@angular/material/icon";
import { CachedSkiArea } from "@/types/config";

@Component({
  selector: "cached-ski-area-item",
  templateUrl: "./cached-ski-area-item.component.html",
  styleUrl: "./cached-ski-area-item.component.scss",
  imports: [MatButtonModule, MatIconModule, MatListModule],
})
export class CachedSkiAreaItem {
  public skiArea = input.required<CachedSkiArea>();
  public disabled = input(false);
  public select = output<void>();
  public delete = output<void>();
  public focus = output<void>();
  public blur = output<void>();

  public date = computed(() => this.skiArea().date.format("YYYY-MM-DD"));
}
