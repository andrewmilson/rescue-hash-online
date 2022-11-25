import { WasmRescueXLIX } from "wasm-rescue";

const DEFAULTS = {
  rescue: {
    stateWidth: 12,
    capacity: 4,
    rounds: 7,
    digestSize: 4,
  },
  fields: [
    {
      modulus: "270497897142230380135924736767050121217",
      description: "1 + 407 * 2^119",
      primitiveElement: "2",
    },
  ],
};

const OTHER = "Other";

const $fieldsDataList = document.getElementById("native-fields");
const $paramsForm = document.getElementById("params-form");
const $primeModulusLabel = document.getElementById("prime-modulus-label");
const $primeModulusInput = document.getElementById("prime-modulus");
const $fieldModulusSelect = document.getElementById("field-modulus");
const $primitiveElementInput = document.getElementById("primitive-element");
const $capacitySizeInput = document.getElementById("capacity-size");
const $stateWidthInput = document.getElementById("state-width");
const $roundsInput = document.getElementById("rounds");
const $digestSizeInput = document.getElementById("digest-size");
const $outputTextarea = document.getElementById("output");

function onFieldChange() {
  const modulus = $fieldModulusSelect.value;

  if (modulus == OTHER) {
    $primeModulusInput.style.display = "block";
    $primeModulusLabel.style.display = "block";
    $primeModulusInput.value = "";
    $primitiveElementInput.value = "";
  } else {
    $primeModulusInput.style.display = "none";
    $primeModulusLabel.style.display = "none";
    $primeModulusInput.value = modulus;
  }

  const fieldEntry = DEFAULTS.fields.find((f) => f.modulus === modulus);
  if (fieldEntry) {
    $primitiveElementInput.value = fieldEntry.primitiveElement;
  }
}

// Initialize
(() => {
  $capacitySizeInput.placeholder = `${DEFAULTS.rescue.capacity} (field elements)`;
  $stateWidthInput.placeholder = `${DEFAULTS.rescue.stateWidth} (field elements)`;
  $roundsInput.placeholder = `${DEFAULTS.rescue.rounds}`;
  $digestSizeInput.placeholder = `${DEFAULTS.rescue.digestSize} (field elements)`;

  for (const field of [...DEFAULTS.fields, { modulus: OTHER }]) {
    var $newOption = document.createElement("option");
    $newOption.value = field.modulus;
    $newOption.textContent = field.modulus;
    if (field.description) {
      $newOption.textContent += ` (${field.description})`;
    }
    $fieldModulusSelect.appendChild($newOption);
  }
  onFieldChange();
})();

$fieldModulusSelect.addEventListener("change", onFieldChange);
$paramsForm.addEventListener("submit", (e) => {
  if (e.preventDefault) e.preventDefault();
  const data = new FormData(e.target);
  const inputs = data
    .get("message")
    .split(/[,\s]+/)
    .filter((n) => n.length);
  const modulus = data.get("prime-modulus");
  const primitiveElement = data.get("primitive-element");

  const capacity = data.get("capacity-size") || DEFAULTS.rescue.capacitySize;
  const stateWidth = data.get("state-width") || DEFAULTS.rescue.stateWidth;
  const rounds = data.get("rounds") || DEFAULTS.rescue.rounds;
  const digestSize = data.get("digest-size") || DEFAULTS.rescue.digestSize;

  const hasher = WasmRescueXLIX.new(
    modulus,
    primitiveElement,
    capacity,
    stateWidth,
    rounds,
    digestSize
  );
  for (const input of inputs) {
    console.log(input);
    hasher.update(input);
  }

  $outputTextarea.value = hasher.finish();
});
