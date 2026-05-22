# ApiV2010AccountIncomingPhoneNumberIncomingPhoneNumberAssignedAddOn

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**sid** | Option<**String**> | The unique string that that we created to identify the resource. | [optional]
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resource. | [optional]
**resource_sid** | Option<**String**> | The SID of the Phone Number to which the Add-on is assigned. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**description** | Option<**String**> | A short description of the functionality that the Add-on provides. | [optional]
**configuration** | Option<**serde_json::Value**> | A JSON string that represents the current configuration of this Add-on installation. | [optional]
**unique_name** | Option<**String**> | An application-defined string that uniquely identifies the resource. It can be used in place of the resource's `sid` in the URL to address the resource. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related resources identified by their relative URIs. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


