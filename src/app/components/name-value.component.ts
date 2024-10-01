import { Component, Input } from "@angular/core";

@Component({
  selector: "name-value",
  standalone: true,
  imports: [],
  templateUrl: "./name-value.component.html",
  styleUrls: ["./name-value.component.css"],
})
export class NameValueComponent {
  @Input()
  public name = "";

  @Input()
  public value = "";
}
