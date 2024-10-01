import { Component, Signal } from "@angular/core";
import { MatProgressBarModule } from "@angular/material/progress-bar";
import { MainMenuComponent } from "@/components/main-menu.component";
import { MapComponent } from "@/components/map.component";
import { SelectionInfoComponent } from "@/components/selection-info.component";
import { ActionsService } from "@/services/actions.service";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [
    MainMenuComponent,
    MapComponent,
    SelectionInfoComponent,
    MatProgressBarModule,
  ],
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.css"],
})
export class AppComponent {
  public loading: Signal<boolean>;

  constructor(private readonly actionsService: ActionsService) {
    this.loading = this.actionsService.loading;
  }
}
