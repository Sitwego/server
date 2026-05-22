# ApiV2010AccountSipSipCredentialList

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The unique id of the [Account](https://www.twilio.com/docs/iam/api/account) that owns this resource. | [optional]
**date_created** | Option<**String**> | The date that this resource was created, given as GMT in [RFC 2822](https://www.php.net/manual/en/class.datetime.php#datetime.constants.rfc2822) format. | [optional]
**date_updated** | Option<**String**> | The date that this resource was last updated, given as GMT in [RFC 2822](https://www.php.net/manual/en/class.datetime.php#datetime.constants.rfc2822) format. | [optional]
**friendly_name** | Option<**String**> | A human readable descriptive text that describes the CredentialList, up to 64 characters long. | [optional]
**sid** | Option<**String**> | A 34 character string that uniquely identifies this resource. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of credentials associated with this credential list. | [optional]
**uri** | Option<**String**> | The URI for this resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


