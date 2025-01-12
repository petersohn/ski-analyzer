import { Component, input } from "@angular/core";
import { CommonModule } from "@angular/common";

@Component({
  selector: "name-value",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./name-value.component.html",
  styleUrls: ["./name-value.component.css"],
})
export class NameValueComponent {
  public name = input("");
  public value = input<string>("");
  public values = input<string[]>([]);
}
