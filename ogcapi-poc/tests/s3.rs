use std::time::Duration;

use aws_smithy_types::{timeout, tristate::TriState};

#[tokio::test]
async fn s3_test() -> Result<(), anyhow::Error> {
    setup_s3_client().await?;
    Ok(())
}

async fn setup_s3_client() -> Result<ogcapi_drivers::s3::S3, anyhow::Error> {
    // Custom timeout
    let api_timeouts = timeout::Api::new()
        .with_call_timeout(TriState::Set(Duration::from_secs(10)))
        .with_call_attempt_timeout(TriState::Set(Duration::from_secs_f32(2.0)));
    let http_timeouts =
        timeout::Http::new().with_connect_timeout(TriState::Set(Duration::from_secs(10)));
    let timeout_config = timeout::Config::new()
        .with_http_timeouts(http_timeouts)
        .with_api_timeouts(api_timeouts);

    let config = aws_config::from_env()
        .timeout_config(timeout_config)
        .load()
        .await;

    let client = aws_sdk_s3::Client::new(&config);

    show_buckets(true, &client, "eu-central-1").await?;

    Ok(ogcapi_drivers::s3::S3 { client })
}

async fn show_buckets(
    strict: bool,
    client: &aws_sdk_s3::Client,
    region: &str,
) -> Result<(), anyhow::Error> {
    let resp = client.list_buckets().send().await?;
    let buckets = resp.buckets().unwrap_or_default();
    let num_buckets = buckets.len();

    let mut in_region = 0;

    for bucket in buckets {
        if strict {
            let r = client
                .get_bucket_location()
                .bucket(bucket.name().unwrap_or_default())
                .send()
                .await?;

            if r.location_constraint().unwrap().as_ref() == region {
                println!("{}", bucket.name().unwrap_or_default());
                in_region += 1;
            }
        } else {
            println!("{}", bucket.name().unwrap_or_default());
        }
    }

    println!();
    if strict {
        println!(
            "Found {} buckets in the {} region out of a total of {} buckets.",
            in_region, region, num_buckets
        );
    } else {
        println!("Found {} buckets in all regions.", num_buckets);
    }

    Ok(())
}
