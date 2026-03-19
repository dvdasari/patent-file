use anyhow::Result;
use tokio::sync::mpsc;

use crate::provider::{LlmProvider, Prompt};

pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }
}

impl LlmProvider for MockProvider {
    fn generate_stream(&self, prompt: Prompt) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(32);

        // Generate mock patent text based on the system prompt hint
        let text = generate_mock_text(&prompt.system);

        tokio::spawn(async move {
            // Stream in chunks to simulate real LLM behavior
            for chunk in text.chars().collect::<Vec<_>>().chunks(50) {
                let s: String = chunk.iter().collect();
                if tx.send(Ok(s)).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });

        Ok(rx)
    }
}

fn generate_mock_text(system_hint: &str) -> String {
    if system_hint.contains("title") {
        "Automated Smart Irrigation Controller with Soil Moisture Sensing".to_string()
    } else if system_hint.contains("field_of_invention") {
        "The present invention relates to the field of agricultural technology, and more particularly to an automated irrigation control system utilizing real-time soil moisture sensing for optimized water distribution.".to_string()
    } else if system_hint.contains("background") {
        "Conventional irrigation systems rely on fixed schedules or manual intervention, leading to significant water wastage and inconsistent crop hydration. Timer-based systems cannot adapt to changing weather conditions or varying soil moisture levels across different zones of a field. Existing sensor-based systems are prohibitively expensive for small-scale farmers and require complex installation procedures.".to_string()
    } else if system_hint.contains("summary") {
        "The present invention provides an automated irrigation controller that continuously monitors soil moisture levels through distributed sensor nodes and dynamically adjusts water delivery to maintain optimal moisture levels. The system comprises a central processing unit, a plurality of wireless soil moisture sensors, and electronically controlled valve actuators connected to the irrigation infrastructure.".to_string()
    } else if system_hint.contains("detailed_description") {
        "In accordance with a preferred embodiment of the present invention, the automated irrigation controller comprises a central control unit housed in a weather-resistant enclosure. The control unit includes a microprocessor, a wireless communication module, a real-time clock, and non-volatile memory for storing irrigation schedules and sensor calibration data.\n\nThe soil moisture sensing network consists of a plurality of sensor nodes distributed across the irrigated area. Each sensor node comprises a capacitive soil moisture sensor, a temperature sensor, a wireless transceiver, and a battery with solar charging capability.\n\nThe valve control subsystem includes electronically actuated solenoid valves connected to the main water supply line. Each valve controls water flow to a designated irrigation zone. The central control unit communicates with the valve actuators through a wired connection using a standard industrial protocol.".to_string()
    } else if system_hint.contains("claims") {
        "1. An automated irrigation control system comprising:\n   a) a central processing unit configured to receive and process sensor data;\n   b) a plurality of wireless soil moisture sensor nodes, each comprising a capacitive moisture sensor and a wireless transceiver;\n   c) a plurality of electronically actuated valve assemblies connected to irrigation supply lines;\n   wherein the central processing unit is configured to dynamically adjust valve states based on real-time soil moisture readings from the sensor nodes.\n\n2. The system of claim 1, wherein each sensor node further comprises a temperature sensor and a solar-powered battery charging circuit.\n\n3. The system of claim 1, wherein the central processing unit implements a predictive algorithm that anticipates irrigation needs based on historical moisture data and weather forecast information.".to_string()
    } else if system_hint.contains("abstract") {
        "An automated irrigation control system that utilizes distributed wireless soil moisture sensors to dynamically regulate water delivery through electronically controlled valves. The system comprises a central processing unit that receives real-time moisture data from sensor nodes and adjusts irrigation schedules to maintain optimal soil moisture levels, thereby reducing water consumption while maximizing crop yield.".to_string()
    } else if system_hint.contains("drawings") {
        "Figure 1 illustrates the overall system architecture of the automated irrigation controller showing the central control unit, sensor network, and valve subsystem.\n\nFigure 2 shows a detailed view of a wireless soil moisture sensor node including the capacitive sensing element, wireless transceiver, and solar charging circuit.".to_string()
    } else {
        "Mock generated text for patent section.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_streams_text() {
        let provider = MockProvider::new();
        let prompt = Prompt {
            system: "Generate title".to_string(),
            user: "Generate a title for an irrigation invention".to_string(),
        };

        let mut rx = provider.generate_stream(prompt).unwrap();
        let mut result = String::new();
        while let Some(chunk) = rx.recv().await {
            result.push_str(&chunk.unwrap());
        }
        assert!(!result.is_empty());
        assert!(result.contains("Irrigation"));
    }
}
