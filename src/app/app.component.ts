import { Component, computed, Signal } from "@angular/core";
import { MatProgressBarModule } from "@angular/material/progress-bar";
import { MainMenuComponent } from "@/components/main-menu.component";
import { MapComponent } from "@/components/map.component";
import { SkiAreaSelectorComponent } from "@/components/ski-area-selector.component";
import { SelectionInfoComponent } from "@/components/selection-info.component";
import { ActionsService } from "@/services/actions.service";
import { CommonModule } from "@angular/common";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [
    MainMenuComponent,
    MapComponent,
    SelectionInfoComponent,
    SkiAreaSelectorComponent,
    MatProgressBarModule,
    CommonModule,
  ],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent {
  public loading: Signal<boolean>;
  public hasSelectableSkiArea: Signal<boolean>;

  constructor(private readonly actionsService: ActionsService) {
    this.loading = this.actionsService.loading;
    this.hasSelectableSkiArea = computed(
      () => this.actionsService.choosableSkiAreas().length != 0,
    );
  }
}
