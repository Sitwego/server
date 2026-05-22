# ApiV2010AccountAuthorizedConnectApp

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the AuthorizedConnectApp resource. | [optional]
**connect_app_company_name** | Option<**String**> | The company name set for the Connect App. | [optional]
**connect_app_description** | Option<**String**> | A detailed description of the Connect App. | [optional]
**connect_app_friendly_name** | Option<**String**> | The name of the Connect App. | [optional]
**connect_app_homepage_url** | Option<**String**> | The public URL for the Connect App. | [optional]
**connect_app_sid** | Option<**String**> | The SID that we assigned to the Connect App. | [optional]
**permissions** | Option<[**Vec<models::AuthorizedConnectAppEnumPermission>**](AuthorizedConnectAppEnumPermission.md)> | The set of permissions that you authorized for the Connect App.  Can be: `get-all` or `post-all`. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


