use serde;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SearchParams {
    pub number: String,
    pub country: String,
    pub phone: String,
    pub email: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct PendingDebt {
    pub total: i64,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDetails {
    payment_type: String,
    amount: f64,
}

impl PaymentDetails {
    fn from_debt(debt: PendingDebt) -> Self {
        let fraction = debt.total % 100;
        let to_pay = match fraction {
            0 => (debt.total / 100) as f64,
            _ => debt.total as f64 / 100f64,
        };
        PaymentDetails { payment_type: "nspk".to_string(), amount: to_pay }
    }
}

#[derive(serde::Deserialize, Debug)]
struct PaymentLink {
    href: Option<String>,
    result: String,
}

pub struct AvtodorClient {
    client: reqwest::Client,
}

impl AvtodorClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder().cookie_store(true).build()?;
        Ok(AvtodorClient { client })
    }

    pub async fn get_debt(
        &self,
        search_params: SearchParams,
    ) -> Result<PendingDebt, Box<dyn std::error::Error>> {
        // Get cookies for client session
        self.client.get("https://pay.avtodor-tr.ru").send().await?;

        let resp = self
            .client
            .post("https://pay.avtodor-tr.ru/check")
            .json(&search_params)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!(
                "Failed to check [code={}]: {}",
                resp.status(),
                resp.text().await?
            )
            .into());
        }

        Ok(resp.json::<PendingDebt>().await?)
    }

    pub async fn get_payment_link(
        &self,
        debt_info: PendingDebt,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let payment_data = PaymentDetails::from_debt(debt_info);
        println!("Payment info: {:#?}", payment_data);
        let request = self
            .client
            .post("https://pay.avtodor-tr.ru/pay")
            .json(&payment_data)
            .build()?;

        let resp = self.client.execute(request).await?;
        if !resp.status().is_success() {
            return Err(format!(
                "Failed to get payment link [code={}]",
                resp.status()
            )
            .into());
        }
        let resp_data = resp.json::<PaymentLink>().await?;
        if resp_data.result == "error" {
            return Err(
                "Failed to get paymnet link, avtodor returned error".into()
            );
        };
        match resp_data.href {
            Some(link) => Ok(link),
            None => {
                Err(format!("No link in response {}", resp_data.result).into())
            }
        }
    }
}
