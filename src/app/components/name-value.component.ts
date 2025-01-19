import { Component, input, computed } from "@angular/core";
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

  public singleValue = computed(() => {
    const value = this.value();
    if (value !== "") {
      return value;
    }
    const values = this.values();
    if (values.length === 1) {
      return values[0];
    }
    return "";
  });

  public valueList = computed(() => {
    const values = this.values();
    if (values.length === 1 && this.value() === "") {
      return [];
    }
    return values;
  });
}
