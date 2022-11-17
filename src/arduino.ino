void setup() {
  // put your setup code here, to run once:
  Serial.begin(115200);
  while (!Serial) {;}
}

int analog_pin = A0;
int value;
void loop() {
  // put your main code here, to run repeatedly:
  if (Serial.available() > 0) {
    while (Serial.available() > 0) {Serial.read();}
    int data = analogRead(analog_pin);
    Serial.print(data);
  }
}
