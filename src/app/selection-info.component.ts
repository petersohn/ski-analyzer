import { Component, Signal, computed } from "@angular/core";
import { CommonModule } from "@angular/common";
import { MatCardModule } from "@angular/material/card";
import { MapService } from "./map.service";
import { Lift, Piste } from "./types/skiArea";
import { NameValueComponent } from "./name-value.component";

const liftTypes: { [type: string]: string } = {
  cable_car: "Cable car",
  gondola: "Gondola",
  mixed_lift: "Mixed lfit",
  chair_lift: "Chairlift",
  drag_lift: "Draglift",
  "t-bar": "T-bar",
  "j-bar": "J-bar",
  platter: "Platter",
  rope_tow: "Rope tow",
  magic_carpet: "Magic carpet",
  zip_line: "Zipline",
};

@Component({
  selector: "selection-info",
  standalone: true,
  imports: [CommonModule, MatCardModule, NameValueComponent],
  templateUrl: "./selection-info.component.html",
  styleUrls: ["./selection-info.component.css"],
})
export class SelectionInfoComponent {
  public selectedPiste: Signal<Piste | undefined>;
  public selectedLift: Signal<Lift | undefined>;

  constructor(private readonly mapService: MapService) {
    this.selectedPiste = this.mapService.selectedPiste;
    this.selectedLift = this.mapService.selectedLift;
  }

  public pisteName = computed(() => this.getName(this.selectedPiste()));
  public liftName = computed(() => this.getName(this.selectedLift()));
  public liftType = computed(() => liftTypes[this.selectedLift()?.type ?? ""]);

  private getName(input?: { ref: string; name: string }) {
    if (!input) {
      return "";
    }

    if (input.ref && input.name) {
      return `[${input.ref}] ${input.name}`;
    } else {
      return input.name || input.ref || "";
    }
  }
}
