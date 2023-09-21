use {
    aws_sdk_s3::{
        error::SdkError,
        operation::get_object::GetObjectError,
        primitives::ByteStreamError,
        Client as S3Client,
    },
    bytes::Bytes,
    maxminddb::geoip2::City,
    std::{error::Error, net::IpAddr, sync::Arc},
};

#[macro_use]
extern crate justerror;

#[derive(Debug, Clone)]
pub struct GeoData {
    pub continent: Option<Arc<str>>,
    pub country: Option<Arc<str>>,
    pub region: Option<Vec<String>>,
    pub city: Option<Arc<str>>,
}

pub trait GeoIpReader<E: Error> {
    fn lookup_geo_data(&self, addr: IpAddr) -> Result<GeoData, E>;
}

#[Error]
pub enum MaxMindGeoIpError {
    GetObject(#[from] SdkError<GetObjectError>),
    ByteStream(#[from] ByteStreamError),
    MaxMindDB(#[from] maxminddb::MaxMindDBError),
}

pub struct MaxMindGeoIpReader {
    reader: Arc<maxminddb::Reader<Bytes>>,
}

impl MaxMindGeoIpReader {
    pub async fn from_aws_s3(
        s3_client: &S3Client,
        bucket: impl Into<String>,
        key: impl Into<String>,
    ) -> Result<Self, MaxMindGeoIpError> {
        let s3_object = s3_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
        let geo_data = s3_object.body.collect().await?.into_bytes();

        Self::from_buffer(geo_data)
    }

    pub fn from_buffer(buffer: Bytes) -> Result<Self, MaxMindGeoIpError> {
        let reader = maxminddb::Reader::from_source(buffer)?;
        Ok(Self {
            reader: Arc::new(reader),
        })
    }
}

impl GeoIpReader<MaxMindGeoIpError> for MaxMindGeoIpReader {
    fn lookup_geo_data(&self, addr: IpAddr) -> Result<GeoData, MaxMindGeoIpError> {
        let lookup_data = self.reader.lookup::<City>(addr)?;

        Ok(GeoData {
            continent: lookup_data
                .continent
                .and_then(|continent| continent.code.map(Into::into)),
            country: lookup_data
                .country
                .and_then(|country| country.iso_code.map(Into::into)),
            region: lookup_data.subdivisions.map(|divs| {
                divs.into_iter()
                    .filter_map(|div| div.iso_code)
                    .map(Into::into)
                    .collect()
            }),
            city: lookup_data
                .city
                .and_then(|city| city.names)
                .and_then(|city_names| city_names.get("en").copied().map(Into::into)),
        })
    }
}
