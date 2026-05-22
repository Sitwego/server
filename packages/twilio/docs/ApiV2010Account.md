# ApiV2010Account

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**auth_token** | Option<**String**> | The authorization token for this account. This token should be kept a secret, so no sharing. | [optional]
**date_created** | Option<**String**> | The date that this account was created, in GMT in RFC 2822 format | [optional]
**date_updated** | Option<**String**> | The date that this account was last updated, in GMT in RFC 2822 format. | [optional]
**friendly_name** | Option<**String**> | A human readable description of this account, up to 64 characters long. By default the FriendlyName is your email address. | [optional]
**owner_account_sid** | Option<**String**> | The unique 34 character id that represents the parent of this account. The OwnerAccountSid of a parent account is it's own sid. | [optional]
**sid** | Option<**String**> | A 34 character string that uniquely identifies this resource. | [optional]
**status** | Option<[**models::AccountEnumStatus**](AccountEnumStatus.md)> |  | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A Map of various subresources available for the given Account Instance | [optional]
**r#type** | Option<[**models::AccountEnumType**](AccountEnumType.md)> |  | [optional]
**uri** | Option<**String**> | The URI for this resource, relative to `https://api.twilio.com` | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


